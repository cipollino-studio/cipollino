
use std::{fs, path::PathBuf, sync::{Arc, Mutex}};

use crate::{audio::AudioController, export::export_options::ExportOptionsDialog, panels, project::{graphic::Graphic, obj::{obj_list::ObjListTrait, ObjPtr}}, renderer::scene::SceneRenderer};

use self::{clipboard::Clipboard, dialog::{DialogManager, DialogsToOpen}, dropped_files::handle_dropped_files, keybind::{Keybind, RedoKeybind, UndoKeybind}, prefs::{prefs_dialog::PrefsDialog, UserPrefs}, splash_screen::SplashScreen, state::EditorState, toasts::Toasts};

pub mod selection;
pub mod clipboard;
pub mod state;
pub mod dialog;
pub mod splash_screen;
pub mod new_project;
pub mod prefs;
pub mod dropped_files;
pub mod toasts;
pub mod keybind;

pub struct Editor {
    state: Arc<Mutex<EditorState>>,
    panels: panels::PanelManager,
    dialog: DialogManager,
    prefs: UserPrefs,
    toasts: Toasts,
    config_path: PathBuf,

    project_open: bool,

    audio: Option<AudioController>,
    prev_open_graphic: ObjPtr<Graphic>,
    prev_playing: bool
}

pub struct EditorSystems<'a> {
    pub gl: &'a Arc<glow::Context>,
    pub renderer: &'a mut SceneRenderer,
    pub dialog: DialogsToOpen,
    pub prefs: &'a mut UserPrefs,
    pub toasts: &'a mut Toasts,
}

impl Editor {
    
    pub fn new() -> Self {
        let config_path = directories::ProjectDirs::from("com", "Cipollino", "Cipollino").unwrap().config_dir().to_owned();
        let _ = fs::create_dir_all(config_path.clone());

        let panels = if let Ok(data) = std::fs::read(config_path.join("dock.json")) {
            if let Ok(panels) = serde_json::from_slice::<panels::PanelManager>(data.as_slice()) {
                panels
            } else {
                panels::PanelManager::new()
            }
        } else {
            panels::PanelManager::new()
        };
        let state = Arc::new(Mutex::new(EditorState::new())); 

        let mut dialog = DialogManager::new();
        let mut dialogs_to_open = DialogsToOpen::new();
        dialogs_to_open.open_dialog(SplashScreen::new());
        dialog.open_dialogs(dialogs_to_open);

        let prefs = UserPrefs::new(config_path.join("prefs.json"));

        let mut toasts = Toasts::new();
        let audio = match AudioController::new() {
            Ok(audio) => Some(audio),
            Err(msg) => { 
                toasts.error_toast(msg);
                None
            },
        };

        let res = Self {
            state: state.clone(),
            panels,
            dialog,
            prefs,
            toasts: toasts,
            config_path,
            project_open: false,

            audio,
            prev_open_graphic: ObjPtr::null(),
            prev_playing: false
        };
        
        res
    }

    pub fn render(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, scene_renderer: &mut Option<SceneRenderer>) {

        let state = self.state.clone();
        let state = &mut *state.lock().unwrap(); 

        if !self.project_open && state.project.save_path.to_str().unwrap().len() > 0 {
            self.project_open = true;
        }

        let editor_disabled = self.editor_disabled();
        
        if let Some(audio) = &mut self.audio {
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

        // Menu bar
        egui::TopBottomPanel::top("MenuBar").show(ctx, |ui| {
            self.menu_bar(ui, state);
            self.shortcuts(ui, state);
        });

        // Setup systems
        let gl = frame.gl().unwrap();
        if scene_renderer.is_none() {
            *scene_renderer = Some(SceneRenderer::new(gl));
        }
        let mut systems = EditorSystems {
            gl,
            renderer: scene_renderer.as_mut().unwrap(),
            dialog: DialogsToOpen::new(),
            prefs: &mut self.prefs,
            toasts: &mut self.toasts,
        }; 

        // Panels
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |_ui| {
                self.panels.render(ctx, !editor_disabled, state, &mut systems);
            });

        self.dialog.render(ctx, state, &mut systems);

        let _ = std::fs::write(self.config_path.join("dock.json"), serde_json::json!(self.panels).to_string());

        systems.toasts.render(ctx);

        if self.project_open {
            state.project.save(&mut |msg| {
                systems.toasts.error_toast(msg);
            });
        }

        if state.project.graphics.mutated() || state.project.layers.mutated() || state.project.sound_instances.mutated()
           || self.prev_open_graphic != state.open_graphic || self.prev_playing != state.playing {
            set_audio_data(state, &mut self.audio); 
        }
        self.prev_open_graphic = state.open_graphic; 
        self.prev_playing = state.playing;

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
            if let Some(audio) = &mut self.audio {
                let audio_state = audio.state.clone();
                let audio_state = &mut *audio_state.lock().unwrap();
                audio_state.time = state.time;
            }
        }

        handle_dropped_files(ctx, state, &mut systems);

        self.dialog.open_dialogs(systems.dialog);

    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        egui::menu::bar(ui, |ui| {
            ui.set_enabled(!self.editor_disabled());
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().add_filter("Cipollino Project File", &["cip"]).pick_file() {
                        *state = EditorState::load_project(path, &mut self.toasts);
                        return;
                    }
                    ui.close_menu();
                }
                if ui.button("Export").clicked() {
                    let mut dialogs = DialogsToOpen::new();
                    dialogs.open_dialog(ExportOptionsDialog::new());
                    self.dialog.open_dialogs(dialogs);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Preferences").clicked() {
                    let mut dialogs_to_open = DialogsToOpen::new();
                    dialogs_to_open.open_dialog(PrefsDialog::new());
                    self.dialog.open_dialogs(dialogs_to_open);
                    ui.close_menu();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(
                    state.actions.can_undo(),
                    egui::Button::new("Undo").shortcut_text(ui.ctx().format_shortcut(&self.prefs.get::<UndoKeybind>()))).clicked() {
                    state.actions.undo(&mut state.project);
                }
                if ui.add_enabled(
                    state.actions.can_redo(),
                    egui::Button::new("Redo").shortcut_text(ui.ctx().format_shortcut(&self.prefs.get::<RedoKeybind>()))).clicked() {
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
        if UndoKeybind::consume(ui, &mut self.prefs) {
            state.pause();
            state.reset_tool();
            state.actions.undo(&mut state.project);
        }
        if RedoKeybind::consume(ui, &mut self.prefs) {
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

    }

    fn editor_disabled(&self) -> bool {
        !self.project_open
    }

}

fn set_audio_data(state: &EditorState, audio_controller: &mut Option<AudioController>) {
    if let Some(audio) = audio_controller {
        let audio_state = audio.state.clone();
        if let Some(new_state) = state.get_audio_state(state.open_graphic) {
            *audio_state.lock().unwrap() = new_state;
        }
    }
}