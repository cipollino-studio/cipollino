
use std::{sync::{Arc, Mutex}, rc::Rc, cell::RefCell, io::Write, path::PathBuf};

use crate::{panels::{self, timeline::{new_frame, prev_keyframe, next_keyframe}, tools::{pencil::Pencil, Tool, select::Select}}, project::{Project, graphic::Graphic, action::{ActionManager, Action}}, renderer::scene::SceneRenderer, export::Export};
use egui::Modifiers;

pub struct EditorRenderer {
    pub gl_ctx: Arc<Mutex<Option<Arc<glow::Context>>>>,
    renderer: Option<SceneRenderer>,
}

impl EditorRenderer {

    pub fn new() -> Self {
        Self {
            renderer: None,
            gl_ctx: Arc::new(Mutex::new(None))
        }
    }

    pub fn use_renderer<F>(&mut self, f: F) where F: FnOnce(&Arc<glow::Context>, &mut SceneRenderer) {
        if let Some(gl) = self.gl_ctx.lock().unwrap().as_ref() {
            if let None = self.renderer {
                self.renderer = Some(SceneRenderer::new(gl));
            }
            f(gl, self.renderer.as_mut().unwrap());
        }
    }

}

pub struct EditorState {
    // Subsystems
    pub project: Project, 
    pub actions: ActionManager,
    
    // Tools
    pub select: Rc<RefCell<dyn Tool>>,
    pub pencil: Rc<RefCell<dyn Tool>>,
    pub curr_tool: Rc<RefCell<dyn Tool>>,

    // Selections
    pub open_graphic: u64, 
    pub active_layer: u64,
    pub selected_frames: Vec<u64>,
    pub selected_strokes: Vec<u64>,

    // Playback
    pub time: f32,
    pub playing: bool,

    // Display
    pub onion_before: i32,
    pub onion_after: i32,

    // Tool Options
    pub color: glam::Vec3,
    pub stroke_r: f32,
    pub stroke_filled: bool
}

impl EditorState {

    pub fn new_with_project(project: Project) -> Self {
        let select = Rc::new(RefCell::new(Select::new()));
        let pencil = Rc::new(RefCell::new(Pencil::new()));
        Self {
            project: project, 
            actions: ActionManager::new(),
            open_graphic: 0,
            active_layer: 0,
            selected_frames: Vec::new(),
            selected_strokes: Vec::new(),
            time: 0.0,
            playing: false,
            onion_before: 0,
            onion_after: 0,
            select: select.clone(),
            pencil: pencil.clone(),
            curr_tool: select,
            color: glam::Vec3::ZERO,
            stroke_r: 0.05,
            stroke_filled: false
        }
    }

    pub fn new() -> Self {
        let mut res = EditorState::new_with_project(Project::new());
        res.open_graphic = 1;
        res.active_layer = 2;
        res
    }

    pub fn open_graphic(&self) -> Option<&Graphic> {
        self.project.graphics.get(&self.open_graphic)
    }

    pub fn frame_len(&self) -> f32 {
        1.0 / 24.0
    }

    pub fn frame(&self) -> i32 {
        (self.time / (1.0 / 24.0)).floor() as i32
    }

    pub fn copy_shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::C)
    }

    pub fn paste_shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::C)
    }

}

pub struct Editor {
    state: EditorState,
    panels: panels::PanelManager,
    config_path: String,
    save_path: Option<PathBuf>,
    pub renderer: EditorRenderer,
    pub export: Export,
}

impl Editor {
    
    pub fn new() -> Self {
        let config_path = directories::ProjectDirs::from("com", "Cipollino", "Cipollino").unwrap().config_dir().to_str().unwrap().to_owned();
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
            save_path: None,
            export: Export::new(), 
            renderer: EditorRenderer::new(),
        };
        
        res
    }

    pub fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(gfx) = self.state.project.graphics.get(&self.state.open_graphic) {
            if self.state.playing {
                self.state.time += ctx.input(|i| i.stable_dt);
                ctx.request_repaint();
            }
            if self.state.time >= (gfx.data.len as f32) * self.state.frame_len() {
                self.state.time = 0.0;
            }
            if self.state.time < 0.0 {
                self.state.time = ((gfx.data.len - 1) as f32) * self.state.frame_len();
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

            let delete_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::X);

            let save_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::S);

            if ui.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
                self.state.playing = false;
                self.state.curr_tool.clone().borrow_mut().reset(&mut self.state);
                self.state.actions.undo(&mut self.state.project);
            }
            if ui.input_mut(|i| i.consume_shortcut(&redo_shortcut)) {
                self.state.playing = false;
                self.state.curr_tool.clone().borrow_mut().reset(&mut self.state);
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
            if ui.input_mut(|i| i.consume_shortcut(&delete_shortcut)) {
                let mut action = Action::new();
                for frame in &self.state.selected_frames {
                    if let Some(acts) = self.state.project.delete_frame(*frame) {
                        action.add_list(acts);
                    }
                }
                for stroke in &self.state.selected_strokes {
                    if let Some(acts) = self.state.project.delete_stroke(*stroke) {
                        action.add_list(acts);
                    }
                }
                self.state.selected_frames.clear();
                self.state.selected_strokes.clear();
                self.state.actions.add(action);
            }

            if ui.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
                if let Some(path) = self.save_path.clone() {
                    self.save_project(path);
                }
            }

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add_enabled(
                        self.save_path.is_some(),
                    egui::Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&save_shortcut))).clicked() {
                        self.save_project(self.save_path.clone().unwrap());
                    }
                    if ui.button("Save As").clicked() {
                        if let Some(mut path) = rfd::FileDialog::new().save_file() {
                            path.set_extension("cip");
                            self.save_path = Some(path.clone());
                            self.save_project(path);
                        }
                        ui.close_menu();
                    }
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("Cipollino Project File", &["cip"]).pick_file() {
                            if let Ok(file) = std::fs::File::open(path.clone()) {
                                let reader = std::io::BufReader::new(file);
                                let proj = serde_json::from_reader(reader).unwrap(); 
                                self.state = EditorState::new_with_project(proj);
                                self.save_path = Some(path);
                            }
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
                self.panels.render(ctx, self.export.exporting.is_none(), &mut self.state, &mut self.renderer);
            });

        self.export.render(ctx, &mut self.state, &mut self.renderer);

        let _ = std::fs::write(self.config_path.clone() + "/dock.json", serde_json::json!(self.panels).to_string());

    }

    pub fn save_project(&self, path: PathBuf) {
        if let Ok(file) = std::fs::File::create(path.clone()) {
            let mut writer = std::io::BufWriter::new(file);
            serde_json::to_writer(&mut writer, &self.state.project).expect("Could not save");
            let _ = writer.flush();
        }
    }

}
