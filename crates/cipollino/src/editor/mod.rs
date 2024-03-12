
use std::{fs, path::PathBuf, sync::{Arc, Mutex, RwLock}};

use crate::{audio::AudioController, export::Export, panels::{self, timeline::{new_frame, next_keyframe, prev_keyframe}}, project::{action::ActionManager, graphic::Graphic, layer::{Layer, LayerKind}, obj::ObjPtr, palette::Palette, stroke::{Stroke, StrokeColor}, Project}, renderer::scene::SceneRenderer, tools::{bucket::Bucket, color_picker::ColorPicker, line::Line, pencil::Pencil, select::Select, Tool}};
use egui::Modifiers;
use egui_toast::ToastOptions;

use self::{clipboard::Clipboard, selection::Selection};

pub mod selection;
pub mod clipboard;

pub struct EditorRenderer<'a> {
    pub gl: &'a Arc<glow::Context>,
    pub renderer: &'a mut SceneRenderer,
}

pub struct EditorState {
    // Subsystems
    pub project: Project, 
    pub actions: ActionManager,
    pub toasts: egui_toast::Toasts,
   
    // Tools
    pub tools: Vec<Arc<RwLock<dyn Tool + Send + Sync>>>,
    pub curr_tool: Arc<RwLock<dyn Tool + Send + Sync>>,

    // Selections
    pub open_graphic: ObjPtr<Graphic>,
    pub open_palette: ObjPtr<Palette>,
    pub active_layer: ObjPtr<Layer>,
    pub selection: selection::Selection,

    // Clipboard
    pub clipboard: clipboard::Clipboard,

    // Playback
    pub time: i64, // Measured in samples
    pub playing: bool,

    // Display
    pub onion_before: i32,
    pub onion_after: i32,

    // Tool Options
    pub color: StrokeColor,
    pub stroke_r: f32,
    pub stroke_filled: bool,

    // Misc
    pub pasted: bool
}

impl EditorState {

    pub fn new_with_project(project: Project) -> Self {
        let select = Arc::new(RwLock::new(Select::new()));
        let pencil = Arc::new(RwLock::new(Pencil::new()));
        let bucket = Arc::new(RwLock::new(Bucket::new()));
        let color_picker = Arc::new(RwLock::new(ColorPicker::new()));
        let line = Arc::new(RwLock::new(Line::new()));
        Self {
            project: project, 
            actions: ActionManager::new(),
            toasts: egui_toast::Toasts::default().anchor(egui::Align2::RIGHT_BOTTOM, egui::pos2(-10.0, -10.0)),
            open_graphic: ObjPtr::null(),
            open_palette: ObjPtr::null(),
            active_layer: ObjPtr::null(),
            selection: selection::Selection::None,
            clipboard: clipboard::Clipboard::None,
            time: 0,
            playing: false,
            onion_before: 0,
            onion_after: 0,
            tools: vec![select.clone(), pencil, bucket, color_picker, line],
            curr_tool: select,
            color: StrokeColor::Color(glam::vec4(0.0, 0.0, 0.0, 1.0)),
            stroke_r: 5.0,
            stroke_filled: false,
            pasted: false
        }
    }

    pub fn new() -> Self {
        EditorState::new_with_project(Project::new())
    }

    pub fn visible_strokes(&self) -> Vec<ObjPtr<Stroke>> {
        let mut res = Vec::new();
        if let Some(graphic) = self.project.graphics.get(self.open_graphic) {
            for layer in &graphic.layers {
                let layer = layer.get(&self.project);
                if !layer.show || layer.kind != LayerKind::Animation {
                    continue;
                }
                if let Some(frame) = layer.get_frame_at(&self.project, self.frame()) {
                    for stroke in &frame.get(&self.project).strokes {
                        res.push(stroke.make_ptr());
                    }
                }
            }
        }
        res
    }

    pub fn pause(&mut self) {
        self.selection = Selection::None;
        self.playing = false;
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn frame_rate(&self) -> f32 {
        24.0
    }

    pub fn frame_len(&self) -> f32 {
        1.0 / self.frame_rate() 
    }

    pub fn sample_rate(&self) -> f32 {
        44100.0
    }

    pub fn sample_len(&self) -> f32 {
        1.0 / self.sample_rate()
    }

    pub fn time_secs(&self) -> f32 {
        self.time as f32 * self.sample_len()
    }

    pub fn frame(&self) -> i32 {
        (self.time_secs() / self.frame_len()).floor() as i32
    }  

    pub fn delete_shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::X)
    }

    pub fn reset_tool(&mut self) {
        self.curr_tool.clone().write().unwrap().reset(self);
    }

    pub fn with<F>(&mut self, f: F) where F: FnOnce(&mut Self) {
        f(self)
    }

}

pub struct Editor {
    state: Arc<Mutex<EditorState>>,
    _audio: AudioController,
    panels: panels::PanelManager,
    config_path: String,
    pub export: Export,
}

impl Editor {
    
    pub fn new() -> Self {
        let config_path = directories::ProjectDirs::from("com", "Cipollino", "Cipollino").unwrap().config_dir().to_str().unwrap().to_owned();
        let _ = fs::create_dir(config_path.clone());
        let panels = if let Ok(data) = std::fs::read(config_path.clone() + "/dock.json") {
            if let Ok(panels) = serde_json::from_slice::<panels::PanelManager>(data.as_slice()) {
                panels
            } else {
                panels::PanelManager::new()
            }
        } else {
            panels::PanelManager::new()
        };
        let state = Arc::new(Mutex::new(EditorState::new())); 
        let res = Self {
            state: state.clone(),
            _audio: AudioController::new(state),
            panels,
            config_path,
            export: Export::new(), 
        };
        
        res
    }

    pub fn render(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, scene_renderer: &mut Option<SceneRenderer>) {
        let gl = frame.gl().unwrap();
        if scene_renderer.is_none() {
            *scene_renderer = Some(SceneRenderer::new(gl));
        }
        let mut renderer = EditorRenderer {
            gl,
            renderer: scene_renderer.as_mut().unwrap()
        }; 

        self.state.lock().unwrap().with(|state| {
            if let Some(gfx) = state.project.graphics.get(state.open_graphic) {
                if state.playing {
                    ctx.request_repaint();
                }
                if state.time_secs() >= (gfx.len as f32) * state.frame_len() {
                    state.time = 0;
                }
                if state.time < 0 {
                    state.time = (((gfx.len - 1) as f32) * state.frame_len() / state.sample_len()).floor() as i64;
                }
            }
        });

        egui::TopBottomPanel::top("MenuBar").show(ctx, |ui| {

            ui.set_enabled(self.export.exporting.is_none());

            let undo_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::Z);
            let redo_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::Y);
            let play_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Space);
            let frame_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::K);
            
            let prev_frame_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Q);
            let next_frame_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::W);
            let prev_keyframe_shortcut = egui::KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::Q);
            let next_keyframe_shortcut = egui::KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::W);

            let save_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::S);

            self.state.lock().unwrap().with(|mut state| {
                if ui.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
                    state.pause();
                    state.reset_tool();
                    state.actions.undo(&mut state.project);
                }
                if ui.input_mut(|i| i.consume_shortcut(&redo_shortcut)) {
                    state.pause();
                    state.reset_tool();
                    state.actions.redo(&mut state.project);
                }
                if ui.input_mut(|i| i.consume_shortcut(&play_shortcut)) {
                    if state.playing {
                        state.pause();
                    } else {
                        state.play();
                    }
                }
                if ui.input_mut(|i| i.consume_shortcut(&frame_shortcut)) {
                    state.pause();
                    new_frame(&mut state);
                }
                if ui.input_mut(|i| i.consume_shortcut(&prev_frame_shortcut)) {
                    state.pause();
                    state.time = (((state.frame() - 1) as f32) * state.frame_len() / state.sample_len()).floor() as i64 + 1;
                }
                if ui.input_mut(|i| i.consume_shortcut(&next_frame_shortcut)) {
                    state.pause();
                    state.time = (((state.frame() + 1) as f32) * state.frame_len() / state.sample_len()).floor() as i64 + 1;
                }
                if ui.input_mut(|i| i.consume_shortcut(&prev_keyframe_shortcut)) {
                    state.pause();
                    prev_keyframe(&mut state); 
                }
                if ui.input_mut(|i| i.consume_shortcut(&next_keyframe_shortcut)) {
                    state.pause();
                    next_keyframe(&mut state); 
                }

                state.pasted = false;
                for event in ui.input(|i| i.filtered_events(&egui::EventFilter::default())) {
                    match event {
                        egui::Event::Copy => {
                            state.clipboard = Clipboard::from_selection(&state.selection, &mut state.project);
                            if !state.selection.is_empty() {
                                ui.output_mut(|o| o.copied_text = "_".to_owned());
                            }
                        },
                        egui::Event::Paste(_) => {
                            state.pasted = true;
                        },
                        _ => ()
                    }
                }
            });

            if ui.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
                self.save();
            }

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add(egui::Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&save_shortcut))).clicked() {
                        self.save();
                        ui.close_menu();
                    }
                    if ui.button("Save As").clicked() {

                        self.save_as();
                        ui.close_menu();
                    }
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("Cipollino Project File", &["cip"]).pick_file() {
                            *self.state.lock().unwrap() = EditorState::new_with_project(Project::load(path));
                            return;
                        }
                        ui.close_menu();
                    }
                    if ui.button("Export").clicked() {
                        self.export.dialog_open = true;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    self.state.lock().unwrap().with(|state| {
                        if ui.add_enabled(
                            state.actions.can_undo(),
                            egui::Button::new("Undo").shortcut_text(ui.ctx().format_shortcut(&undo_shortcut))).clicked() {
                            state.actions.undo(&mut state.project);
                        }
                        if ui.add_enabled(
                            state.actions.can_redo(),
                            egui::Button::new("Redo").shortcut_text(ui.ctx().format_shortcut(&redo_shortcut))).clicked() {
                            state.actions.redo(&mut state.project);
                        }
                    });
                });
                ui.menu_button("View", |ui| {
                    ui.menu_button("Add Panel", |ui| {
                        if ui.button("Assets").clicked() {
                            self.panels.add_panel(panels::Panel::Assets(panels::assets::AssetsPanel::new()));
                        }
                        if ui.button("Timeline").clicked() {
                            self.panels.add_panel(panels::Panel::Timeline(panels::timeline::TimelinePanel::new()));
                        }
                        if ui.button("Scene").clicked() {
                            self.panels.add_panel(panels::Panel::Scene(panels::scene::ScenePanel::new()));
                        }
                        if ui.button("Tool Options").clicked() {
                            self.panels.add_panel(panels::Panel::Tool(panels::tool::ToolPanel::new()));
                        }
                        if ui.button("Colors").clicked() {
                            self.panels.add_panel(panels::Panel::Color(panels::colors::ColorPanel::new()));
                        }
                    })
                });
            });

        });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |_ui| {
                self.state.lock().unwrap().with(|state| {
                    self.panels.render(ctx, self.export.exporting.is_none(), state, &mut renderer);
                });
            });

        self.state.lock().unwrap().with(|state| {
            self.export.render(ctx, state, &mut renderer);
        });

        let _ = std::fs::write(self.config_path.clone() + "/dock.json", serde_json::json!(self.panels).to_string());

        self.state.lock().unwrap().with(|state| {
            state.toasts.show(ctx);
            state.project.garbage_collect_objs();
        });

    }

    fn save(&mut self) {
        self.state.lock().unwrap().with(|state| {
            save(state);
        });
    }

    fn save_as(&mut self) {
        self.state.lock().unwrap().with(|state| {
            save_as(state);
        });
    }


}

fn save(state: &mut EditorState) {
    if let Some(path) = &state.project.save_path {
        save_project(state, path.clone());
    } else {
        save_as(state);
    }
}

pub fn save_as(state: &mut EditorState) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        if let Ok(dir) = fs::read_dir(path.clone()) {
            if dir.count() == 0 {
                save_project(state, path);
            } else {
                state.toasts.add(egui_toast::Toast {
                    kind: egui_toast::ToastKind::Error,
                    text: "Cannot save project to non-empty directory".into(),
                    options: ToastOptions::default().show_progress(false),
                });
            }
        }
    }
}

pub fn save_project(state: &mut EditorState, path: PathBuf) {
    state.project.save(path); 
}
