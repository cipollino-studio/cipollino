
use std::{sync::{Arc, Mutex}, rc::Rc, cell::RefCell};

use crate::{panels::{self, timeline::{new_frame, prev_keyframe, next_keyframe}, tools::{pencil::Pencil, Tool}}, project::{Project, graphic::Graphic, action::ActionManager}, renderer::scene::SceneRenderer};
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
    pub project: Project, 
    pub actions: ActionManager,
    pub open_graphic: Option<u64>,
    pub active_layer: u64,
    pub time: f32,
    pub playing: bool,
    pub renderer: EditorRenderer,
    
    pub pencil: Rc<RefCell<dyn Tool>>,
    pub curr_tool: Rc<RefCell<dyn Tool>>
}

impl EditorState {

    pub fn new() -> Self {
        let pencil = Rc::new(RefCell::new(Pencil::new()));
        Self {
            project: Project::new(),
            actions: ActionManager::new(),
            open_graphic: None,
            active_layer: 0,
            time: 0.0,
            playing: false,
            renderer: EditorRenderer::new(),
            pencil: pencil.clone(),
            curr_tool: pencil
        }
    }

    pub fn open_graphic(&self) -> Option<&Graphic> {
        if let Some(key) = self.open_graphic {
            self.project.graphics.get(&key)
        } else {
            None
        }
    }

    pub fn frame_len(&self) -> f32 {
        1.0 / 24.0
    }

    pub fn frame(&self) -> i32 {
        (self.time / (1.0 / 24.0)).floor() as i32
    }

    

}

pub struct Editor {
    state: EditorState,
    panels: panels::PanelManager,
    config_path: String,
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
        };
        
        res
    }

    pub fn render(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(open_gfx) = self.state.open_graphic {
            if let Some(gfx) = self.state.project.graphics.get(&open_gfx) {
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
        }

        egui::TopBottomPanel::top("MenuBar").show(ctx, |ui| {

            let undo_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::Z);
            let redo_shortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::Y);
            let play_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Space);
            let frame_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::K);
            
            let prev_frame_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Q);
            let next_frame_shortcut = egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::W);
            let prev_keyframe_shortcut = egui::KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::Q);
            let next_keyframe_shortcut = egui::KeyboardShortcut::new(Modifiers::SHIFT, egui::Key::W);

            if ui.input_mut(|i| i.consume_shortcut(&undo_shortcut)) {
                self.state.playing = false;
                self.state.actions.undo(&mut self.state.project);
            }
            if ui.input_mut(|i| i.consume_shortcut(&redo_shortcut)) {
                self.state.playing = false;
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

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |_ui| {
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
                    })
                });
            });

        });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |_ui| {
                self.panels.render(ctx, &mut self.state);
            });

        let _ = std::fs::write(self.config_path.clone() + "/dock.json", serde_json::json!(self.panels).to_string());

    }

}
