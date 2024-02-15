
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc, sync::Arc};

use crate::{panels::{self, timeline::{new_frame, prev_keyframe, next_keyframe}, tools::{pencil::Pencil, Tool, select::Select, bucket::Bucket}}, project::{Project, graphic::Graphic, action::ActionManager, obj::ObjPtr, stroke::Stroke, layer::Layer}, renderer::scene::SceneRenderer, export::Export};
use egui::Modifiers;
use egui_toast::ToastOptions;

use self::clipboard::Clipboard;

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
    pub tools: Vec<Rc<RefCell<dyn Tool>>>,
    pub curr_tool: Rc<RefCell<dyn Tool>>,

    // Selections
    pub open_graphic: ObjPtr<Graphic>,
    pub active_layer: ObjPtr<Layer>,
    pub selection: selection::Selection,

    // Clipboard
    pub clipboard: clipboard::Clipboard,

    // Playback
    pub time: f32,
    pub playing: bool,

    // Display
    pub onion_before: i32,
    pub onion_after: i32,

    // Tool Options
    pub color: glam::Vec3,
    pub stroke_r: f32,
    pub stroke_filled: bool,

    // Misc
    pub pasted: bool
}

impl EditorState {

    pub fn new_with_project(project: Project) -> Self {
        let select = Rc::new(RefCell::new(Select::new()));
        let pencil = Rc::new(RefCell::new(Pencil::new()));
        let bucket = Rc::new(RefCell::new(Bucket::new()));
        Self {
            project: project, 
            actions: ActionManager::new(),
            toasts: egui_toast::Toasts::default().anchor(egui::Align2::RIGHT_BOTTOM, egui::pos2(-10.0, -10.0)),
            open_graphic: ObjPtr::null(),
            active_layer: ObjPtr::null(),
            selection: selection::Selection::None,
            clipboard: clipboard::Clipboard::None,
            time: 0.0,
            playing: false,
            onion_before: 0,
            onion_after: 0,
            tools: vec![select.clone(), pencil, bucket],
            curr_tool: select,
            color: glam::Vec3::ZERO,
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
                if !layer.get(&self.project).show {
                    continue;
                }
                if let Some(frame) = layer.get(&self.project).get_frame_at(&self.project, self.frame()) {
                    for stroke in &frame.get(&self.project).strokes {
                        res.push(stroke.make_ptr());
                    }
                }
            }
        }
        res
    }

    pub fn frame_len(&self) -> f32 {
        1.0 / 24.0
    }

    pub fn frame(&self) -> i32 {
        (self.time / (1.0 / 24.0)).floor() as i32
    }  

    pub fn delete_shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::X)
    }

    pub fn reset_tool(&mut self) {
        self.curr_tool.clone().borrow_mut().reset(self);
    }

}

pub struct Editor {
    state: EditorState,
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
        let res = Self {
            state: EditorState::new(),
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

        if let Some(gfx) = self.state.project.graphics.get(self.state.open_graphic) {
            if self.state.playing {
                self.state.time += ctx.input(|i| i.stable_dt);
                ctx.request_repaint();
            }
            if self.state.time >= (gfx.len as f32) * self.state.frame_len() {
                self.state.time = 0.0;
            }
            if self.state.time < 0.0 {
                self.state.time = ((gfx.len - 1) as f32) * self.state.frame_len();
            }
        }


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

            if ui.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
                self.state.playing = false;
                self.state.reset_tool();
                self.state.actions.undo(&mut self.state.project);
            }
            if ui.input_mut(|i| i.consume_shortcut(&redo_shortcut)) {
                self.state.playing = false;
                self.state.reset_tool();
                self.state.actions.redo(&mut self.state.project);
            }
            if ui.input_mut(|i| i.consume_shortcut(&play_shortcut)) {
                self.state.playing = !self.state.playing;
            }
            if ui.input_mut(|i| i.consume_shortcut(&frame_shortcut)) {
                self.state.playing = false;
                new_frame(&mut self.state);
            }
            if ui.input_mut(|i| i.consume_shortcut(&prev_frame_shortcut)) {
                self.state.playing = false;
                self.state.time = ((self.state.frame() - 1) as f32) * self.state.frame_len();
            }
            if ui.input_mut(|i| i.consume_shortcut(&next_frame_shortcut)) {
                self.state.playing = false;
                self.state.time = ((self.state.frame() + 1) as f32) * self.state.frame_len();
            }
            if ui.input_mut(|i| i.consume_shortcut(&prev_keyframe_shortcut)) {
                self.state.playing = false;
                prev_keyframe(&mut self.state); 
            }
            if ui.input_mut(|i| i.consume_shortcut(&next_keyframe_shortcut)) {
                self.state.playing = false;
                next_keyframe(&mut self.state); 
            }

            self.state.pasted = false;
            for event in ui.input(|i| i.filtered_events(&egui::EventFilter::default())) {
                match event {
                    egui::Event::Copy => {
                        self.state.clipboard = Clipboard::from_selection(&self.state.selection, &mut self.state.project);
                        if !self.state.selection.is_empty() {
                            ui.output_mut(|o| o.copied_text = "_".to_owned());
                        }
                    },
                    egui::Event::Paste(_) => {
                        self.state.pasted = true;
                    },
                    _ => ()
                }
            }

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
                            self.state = EditorState::new_with_project(Project::load(path));
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
                    if ui.add_enabled(
                        self.state.actions.can_undo(),
                        egui::Button::new("Undo").shortcut_text(ui.ctx().format_shortcut(&undo_shortcut))).clicked() {
                        self.state.actions.undo(&mut self.state.project);
                    }
                    if ui.add_enabled(
                        self.state.actions.can_redo(),
                        egui::Button::new("Redo").shortcut_text(ui.ctx().format_shortcut(&redo_shortcut))).clicked() {
                        self.state.actions.redo(&mut self.state.project);
                    }
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
                    })
                });
            });

        });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |_ui| {
                self.panels.render(ctx, self.export.exporting.is_none(), &mut self.state, &mut renderer);
            });

        self.export.render(ctx, &mut self.state, &mut renderer);

        let _ = std::fs::write(self.config_path.clone() + "/dock.json", serde_json::json!(self.panels).to_string());

        self.state.toasts.show(ctx);

        self.state.project.garbage_collect_objs();

    }

    pub fn save(&mut self) {
        if let Some(path) = &self.state.project.save_path {
            self.save_project(path.clone());
        } else {
            self.save_as();
        }
    }

    pub fn save_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            if let Ok(dir) = fs::read_dir(path.clone()) {
                if dir.count() == 0 {
                    self.save_project(path);
                } else {
                    self.state.toasts.add(egui_toast::Toast {
                        kind: egui_toast::ToastKind::Error,
                        text: "Cannot save project to non-empty directory".into(),
                        options: ToastOptions::default().show_progress(false),
                    });
                }
            }
        }
    }

    pub fn save_project(&mut self, path: PathBuf) {
        self.state.project.save(path); 
    }

}
