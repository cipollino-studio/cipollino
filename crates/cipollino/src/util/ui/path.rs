
use std::{path::PathBuf, str::FromStr};

pub fn path_selector<F>(ui: &mut egui::Ui, path: &mut PathBuf, correct_path: F) 
    where F: Fn(&mut PathBuf) {
    ui.horizontal(|ui| {
        let mut path_as_string = path.to_str().unwrap().to_owned();
        let text_edit_resp = ui.text_edit_singleline(&mut path_as_string);
        *path = PathBuf::from_str(&path_as_string).unwrap();
        if text_edit_resp.lost_focus() {
            correct_path(path);
        }

        if ui.button(egui_phosphor::regular::FOLDER).clicked() {
            if let Some(mut new_path) = rfd::FileDialog::new().save_file() {
                correct_path(&mut new_path);
                *path = new_path;
            }
        }
    });
}
