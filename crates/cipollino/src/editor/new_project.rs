
use std::path::PathBuf;
use unique_type_id::UniqueTypeId;

use crate::{project::Project, util::ui::path::path_selector};

use super::{dialog::Dialog, splash_screen::{push_recent_project, SplashScreen}, state::EditorState, EditorSystems};

#[derive(UniqueTypeId)]
pub struct NewProject {
    user_dirs: directories::UserDirs,
    project_name: String,
    project_location: PathBuf,
    project_fps: f32,
    project_sample_rate: f32 
}

const FPS_OPTIONS: [f32; 5] = [18.0, 24.0, 30.0, 48.0, 60.0];
const SAMPLE_RATE_OPTIONS: [f32; 5] = [44100.0, 48000.0, 88200.0, 96000.0, 192000.0];

pub fn default_project_location() -> PathBuf {
    let user_dirs = directories::UserDirs::new().unwrap();
    let mut default_project_location = user_dirs.document_dir().unwrap().to_owned();
    default_project_location.push("Cipollino Projects");
    default_project_location
}

impl NewProject {

    pub fn new() -> Self {
        let user_dirs = directories::UserDirs::new().unwrap();
        Self {
            user_dirs,
            project_name: "".to_owned(),
            project_location: default_project_location(),
            project_fps: 24.0,
            project_sample_rate: 44100.0
        }
    }

    fn new_project_path(&self) -> PathBuf {
        self.project_location.join(self.project_name.clone())
    }

}

impl Dialog for NewProject {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut super::state::EditorState, systems: &mut EditorSystems) -> bool {
        let mut close_dialog = false;

        let mut project_path_valid = true;

        if ui.add(egui::Label::new(egui_phosphor::regular::ARROW_LEFT).sense(egui::Sense::click())).clicked() {
            systems.dialog.open_dialog(SplashScreen::new());
            close_dialog = true;
        }
        ui.add_space(15.0);

        let right_align_layout = egui::Layout::top_down(egui::Align::RIGHT);
        egui::Grid::new(ui.next_auto_id()).min_col_width(80.0).show(ui, |ui| {
            ui.with_layout(right_align_layout, |ui| {
                ui.label("Project Name:"); 
            });
            ui.add(egui::TextEdit::singleline(&mut self.project_name).hint_text("My Awesome Animation"));
            self.project_name = self.project_name.replace("/", "");
            ui.end_row();

            ui.with_layout(right_align_layout, |ui| {
                ui.label("Location:");
            });
            path_selector(ui, &mut self.project_location, true, |_| {});
            ui.end_row();

            if self.project_name.is_empty() {
                ui.label("");
                ui.label(egui::RichText::new("Project name cannot be empty.").color(ui.style().visuals.error_fg_color));
                ui.end_row();

                ui.label("");
                ui.label("");
                ui.end_row();

                project_path_valid = false;
            } else {
                let project_path = self.new_project_path();
                if let Some(display_path) = pathdiff::diff_paths(project_path.clone(), self.user_dirs.home_dir()) {
                    if project_path.exists() { 
                        ui.label("");
                        ui.label(egui::RichText::new("Folder already exists at:").color(ui.style().visuals.error_fg_color));
                        ui.end_row();

                        ui.label("");
                        ui.allocate_ui(egui::vec2(400.0, ui.available_size_before_wrap().y), |ui| {
                            let text = egui::RichText::new(display_path.to_str().unwrap()).color(ui.style().visuals.error_fg_color);
                            ui.add(egui::Label::new(text).wrap(true));
                        });
                        ui.end_row();

                        project_path_valid = false;
                    } else {
                        ui.label("");
                        ui.label("Project will be saved to:");
                        ui.end_row();

                        ui.label("");
                        ui.allocate_ui(egui::vec2(400.0, ui.available_size_before_wrap().y), |ui| {
                            ui.add(egui::Label::new(display_path.to_str().unwrap()).wrap(true));
                        });
                        ui.end_row();
                    }
                } else {
                    ui.label("");
                    ui.label(egui::RichText::new("Invalid location path.").color(ui.style().visuals.error_fg_color));
                    ui.end_row();

                    ui.label("");
                    ui.label("");
                    ui.end_row();

                    project_path_valid = false;
                }
            }

            ui.add_space(12.0);
            ui.end_row();

            ui.with_layout(right_align_layout, |ui| {
                ui.label("Frame Rate:");
            });
            egui::ComboBox::new(ui.next_auto_id(), "")
                .selected_text(format!("{}", self.project_fps)).show_ui(ui, |ui| {
                    for fps_option in FPS_OPTIONS {
                        ui.selectable_value(&mut self.project_fps, fps_option, format!("{}", fps_option));
                    }
            });
            ui.end_row();

            ui.with_layout(right_align_layout, |ui| {
                ui.label("Sample Rate:");
            });
            egui::ComboBox::new(ui.next_auto_id(), "")
                .selected_text(format!("{}", self.project_sample_rate)).show_ui(ui, |ui| {
                    for sample_rate_option in SAMPLE_RATE_OPTIONS {
                        ui.selectable_value(&mut self.project_sample_rate, sample_rate_option, format!("{}", sample_rate_option));
                    }
            });
            ui.end_row();
        });

        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            if ui.add_enabled(project_path_valid, egui::Button::new("Create!")).clicked() {
                let mut project_path = self.new_project_path();
                project_path.push("proj.cip");
                push_recent_project(systems.prefs, project_path.clone());
                *state = EditorState::new_with_project(Project::create(project_path, self.project_fps, self.project_sample_rate));
                close_dialog = true;
            }
        });

        close_dialog 
    }

    fn title(&self, _: &super::state::EditorState) -> String {
        "".to_owned()
    }
    
    fn show_title(&self) -> bool {
        false
    }

    fn resizable(&self) -> bool {
        false
    }

    fn anchor(&self) -> Option<egui::Align2> {
        Some(egui::Align2::CENTER_CENTER)
    }

    fn margin(&self) -> Option<egui::Margin> {
        Some(egui::Margin::same(20.0))
    }

}
