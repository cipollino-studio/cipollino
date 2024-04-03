
use crate::project::saveload::load::LoadingMetadata;

use super::{state::EditorState, EditorSystems};

pub fn handle_dropped_files(ctx: &egui::Context, state: &mut EditorState, systems: &mut EditorSystems) {

    if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped_files {
            let path = file.path.expect("Path should be set for desktop egui backend.");

            if path.starts_with(state.project.base_path()) {
                continue;
            }

            let file_name = path.file_name().unwrap().to_str().unwrap();
            let mut dest_path = state.project.base_path().join(file_name);
            if dest_path.exists() {
                for i in 1.. {
                    let new_name = if path.is_dir() {
                        format!("{} ({})", path.file_stem().unwrap().to_str().unwrap(), i)
                    } else {
                        format!("{} ({}).{}", path.file_stem().unwrap().to_str().unwrap(), i, path.extension().unwrap().to_str().unwrap())
                    };
                    dest_path = state.project.base_path().join(new_name);
                    if !dest_path.exists() {
                        break;
                    }
                }
            }
            
            let mut metadata = LoadingMetadata::new();
            match std::fs::rename(path, dest_path.clone()) {
                Ok(_) => {
                    state.project.load_file_to_root_folder(dest_path, &mut metadata);
                },
                Err(err) => {
                    systems.toasts.error_toast(format!("Could not move file: {}.", err.to_string()));
                } 
            }
            metadata.display_errors(state, systems.toasts);

        }     

        ctx.request_repaint();
    }
    
}
