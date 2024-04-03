
use std::path::PathBuf;

use unique_type_id::UniqueTypeId;

use super::{dialog::Dialog, new_project::{default_project_location, NewProject}, prefs::{UserPref, UserPrefs}, state::EditorState, EditorSystems};

#[derive(UniqueTypeId)]
pub struct SplashScreen {

}

impl SplashScreen {

    pub fn new() -> Self {
        Self {}
    }

    fn splash_screen_left_side(&mut self, ui: &mut egui::Ui, close_dialog: &mut bool, state: &mut EditorState, systems: &mut EditorSystems) {
        egui::Frame::default().inner_margin(egui::Margin {
            left: ui.available_width() / 6.0,
            right: ui.available_width() / 4.0,
            ..egui::Margin::ZERO
        }).show(ui, |ui| {
            ui.label("");
            if ui.link(format!("{} New Project", egui_phosphor::regular::PLUS)).clicked() { 
                systems.dialog.open_dialog(NewProject::new());
                *close_dialog = true;
            }
            if ui.link(format!("{} Open Project", egui_phosphor::regular::FOLDER)).clicked() { 
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Cipollino Project", &["cip"])
                    .set_directory(default_project_location())
                    .pick_file() {
                        *close_dialog = true;
                        push_recent_project(systems.prefs, path.clone());
                        *state = EditorState::load_project(path, systems.toasts);
                }
            }
        });
    } 

    fn splash_screen_right_side(&mut self, ui: &mut egui::Ui, close_dialog: &mut bool, state: &mut EditorState, systems: &mut EditorSystems) {
        ui.label("Recent Projects");
        ui.add_space(12.0);

        let recent_projects = systems.prefs.get::<RecentProjects>();
        for path in recent_projects {
            let found = path.exists(); 
            let project_name = path.parent().map(|path| path.file_name().unwrap().to_str().unwrap()).unwrap_or("???");
            let label_text = if found {
                egui::RichText::new(project_name)
            } else {
                egui::RichText::new(format!("{} {}", egui_phosphor::regular::WARNING, project_name)).color(ui.visuals().weak_text_color())
            };
            let link = ui.add(egui::Label::new(label_text).sense(egui::Sense::click()))
                .on_hover_text(if found {
                    format!("{}", path.parent().unwrap().to_str().unwrap())
                } else {
                    format!("{} missing.", path.to_str().unwrap())
                });
            if link.clicked() && found {
                push_recent_project(systems.prefs, path.clone());
                *state = EditorState::load_project(path, systems.toasts);
                *close_dialog = true;
            }
        }
    }

}

impl Dialog for SplashScreen {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) -> bool {
        let mut close_dialog = false;

        ui.add(egui::Image::new(egui::include_image!("../../../../res/banner.png")).rounding(egui::Rounding {
            se: 0.0,
            sw: 0.0,
            ..ui.visuals().window_rounding
        }));
        egui::Frame::default().inner_margin(ui.ctx().style().spacing.window_margin).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.small("\"Animation consists of nothing but work!\" - Richard Williams")
            });
            ui.add_space(15.0);
            ui.columns(2, |cols| {
                let left = &mut cols[0];
                self.splash_screen_left_side(left, &mut close_dialog, state, systems); 
               
                let right = &mut cols[1];
                self.splash_screen_right_side(right, &mut close_dialog, state, systems);
            });
            ui.add_space(20.0);
        });

        close_dialog 
    }

    fn title(&self, _state: &super::state::EditorState) -> String {
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
        Some(egui::Margin::ZERO)
    }

}

struct RecentProjects;

impl UserPref for RecentProjects {
    type Type = Vec<PathBuf>;

    fn default() -> Self::Type {
        Vec::new()
    }

    fn name() -> &'static str {
        "recent_projects"
    }
}

pub fn push_recent_project(prefs: &mut UserPrefs, project: PathBuf) {
    let mut recents = prefs.get::<RecentProjects>();    
    if let Some(idx) = recents.iter().position(|other| other == &project) {
        recents.remove(idx);
    }
    recents.insert(0, project);
    prefs.set::<RecentProjects>(recents);
}
