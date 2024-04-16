
use std::path::PathBuf;

use unique_type_id::UniqueTypeId;

use crate::{editor::{dialog::Dialog, state::EditorState, EditorSystems}, util::ui::path::path_selector};

use super::export_progress::ExportProgressDialog;

#[derive(UniqueTypeId)]
pub struct ExportOptionsDialog {
    path: PathBuf
}

impl ExportOptionsDialog {

    pub fn new() -> Self {
        Self {
            path: PathBuf::new()
        }
    }

}

impl Dialog for ExportOptionsDialog {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) -> bool {
        let open_graphic = state.project.graphics.get(state.open_graphic);
        if open_graphic.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("Open a clip to export it.");
            });
            return false;
        }
        let open_graphic = open_graphic.unwrap();
        if !open_graphic.clip {
            ui.centered_and_justified(|ui| {
                ui.label("Cannot export non-clip.");
            });
            return false;
        }
        path_selector(ui, &mut self.path, false, |path| {
            path.set_extension("mp4");
        });
        if ui.button("Export").clicked() {
            match ExportProgressDialog::new(self.path.clone(), state.open_graphic, state, systems) {
                Ok(dialog) => systems.dialog.open_dialog(dialog),
                Err(error) => systems.toasts.error_toast(error),
            }
            return true;
        }

        false
    }

    fn title(&self, _state: &EditorState) -> String {
        "Export".to_owned()
    }

}