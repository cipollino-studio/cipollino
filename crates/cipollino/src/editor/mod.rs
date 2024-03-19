
use std::{fs, path::PathBuf, sync::{Arc, Mutex}};

use crate::{export::Export, panels, project::Project, renderer::scene::SceneRenderer};
use egui::{KeyboardShortcut, Modifiers};

use self::{clipboard::Clipboard, state::{EditorRenderer, EditorState}};

pub mod selection;
pub mod clipboard;
pub mod state;

pub const UNDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::Z);
pub const REDO_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::Y);
pub const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::S);

pub struct Editor {
    state: Arc<Mutex<EditorState>>,
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
            panels,
            config_path,
            export: Export::new(), 
        };
        
        res
    }

    pub fn render(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, scene_renderer: &mut Option<SceneRenderer>) {
        let state = self.state.clone();
        let state = &mut *state.lock().unwrap(); 

        let gl = frame.gl().unwrap();
        if scene_renderer.is_none() {
            *scene_renderer = Some(SceneRenderer::new(gl));
        }
        let mut renderer = EditorRenderer {
            gl,
            renderer: scene_renderer.as_mut().unwrap()
        }; 

        if let Some(audio) = &mut state.audio {
            if let Some(_gfx) = state.project.graphics.get(state.open_graphic) {
                audio.set_playing(state.playing);

                let audio_state = audio.state.clone();
                let audio_state = &mut *audio_state.lock().unwrap();
                state.time = audio_state.time;
            
                if state.playing {
                    ctx.request_repaint();
                }
            } else {
                audio.set_playing(false);
                state.playing = false;
                state.time = 0;
            }
        }

        let initial_time = state.time;

        egui::TopBottomPanel::top("MenuBar").show(ctx, |ui| {
            self.menu_bar(ui, state);
            self.shortcuts(ui, state);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |_ui| {
                self.panels.render(ctx, !self.export.exporting(), state, &mut renderer);
            });

        self.export.render(ctx, state, &mut renderer);

        let _ = std::fs::write(self.config_path.clone() + "/dock.json", serde_json::json!(self.panels).to_string());

        state.toasts.show(ctx);
        state.project.garbage_collect_objs();

        if let Some(gfx) = state.project.graphics.get(state.open_graphic) {
            let gfx_len_in_samples = ((gfx.len as f32) * state.frame_len() * state.sample_rate()) as i64; 
            if state.time < 0 {
                state.time = 0;
            }
            if state.time > gfx_len_in_samples {
                state.time = 0;
            }
        }
        if state.time != initial_time {
            if let Some(audio) = &mut state.audio {
                let audio_state = audio.state.clone();
                let audio_state = &mut *audio_state.lock().unwrap();
                audio_state.time = state.time;
            }
        }

    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        egui::menu::bar(ui, |ui| {
            ui.set_enabled(!self.export.exporting());
            ui.menu_button("File", |ui| {
                if ui.add(egui::Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&SAVE_SHORTCUT))).clicked() {
                    save(state);
                    ui.close_menu();
                }
                if ui.button("Save As").clicked() {
                    save_as(state);
                    ui.close_menu();
                }
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Cipollino Project File", &["cip"]).pick_file() {
                        *state = EditorState::new_with_project(Project::load(path));
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
                    state.actions.can_undo(),
                    egui::Button::new("Undo").shortcut_text(ui.ctx().format_shortcut(&UNDO_SHORTCUT))).clicked() {
                    state.actions.undo(&mut state.project);
                }
                if ui.add_enabled(
                    state.actions.can_redo(),
                    egui::Button::new("Redo").shortcut_text(ui.ctx().format_shortcut(&REDO_SHORTCUT))).clicked() {
                    state.actions.redo(&mut state.project);
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
                    if ui.button("Colors").clicked() {
                        self.panels.add_panel(panels::Panel::Color(panels::colors::ColorPanel::new()));
                    }
                })
            });
        });
    }

    fn shortcuts(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if ui.input_mut(|i| i.consume_shortcut(&UNDO_SHORTCUT)) {
            state.pause();
            state.reset_tool();
            state.actions.undo(&mut state.project);
        }
        if ui.input_mut(|i| i.consume_shortcut(&REDO_SHORTCUT)) {
            state.pause();
            state.reset_tool();
            state.actions.redo(&mut state.project);
        }
        

        state.just_pasted = false;
        for event in ui.input(|i| i.filtered_events(&egui::EventFilter::default())) {
            match event {
                egui::Event::Copy => {
                    state.clipboard = Clipboard::from_selection(&state.selection, &mut state.project);
                    if !state.selection.is_empty() {
                        ui.output_mut(|o| o.copied_text = "_".to_owned());
                    }
                },
                egui::Event::Paste(_) => {
                    state.just_pasted = true;
                },
                _ => ()
            }
        }

        if ui.input_mut(|i| i.consume_shortcut(&SAVE_SHORTCUT)) {
            save(state);
        }
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
                state.error_toast("Cannot save project to non-empty directory.");
            }
        }
    }
}

pub fn save_project(state: &mut EditorState, path: PathBuf) {
    state.project.save(path); 
}
