
use unique_type_id::UniqueTypeId;

use crate::{editor::{dialog::Dialog, keybind::{CenterSceneKeybind, DeleteKeybind, Keybind, NewFrameKeybind, NextFrameKeybind, PlayKeybind, PrevFrameKeybind, RedoKeybind, StepBackKeybind, StepForwardKeybind, UndoKeybind}, state::EditorState, EditorSystems}, tools::{bucket::BucketToolKeybind, color_picker::ColorPickerToolKeybind, line::LineToolKeybind, pencil::PencilToolKeybind, select::SelectToolKeybind}};

#[derive(UniqueTypeId)]
pub struct PrefsDialog {
    keybind_binding: &'static str 
} 

impl PrefsDialog {

    pub fn new() -> Self {
        Self {
            keybind_binding: ""
        }
    }

    fn render_keybind_setting<K: Keybind>(&mut self, ui: &mut egui::Ui, systems: &mut EditorSystems, key_down: &Option<egui::Key>) {

        if self.keybind_binding == K::display_name() {
            if let Some(key) = key_down {
                let new_shortcut = egui::KeyboardShortcut::new(ui.input(|i| i.modifiers), *key);
                systems.prefs.set::<K>(new_shortcut);
                self.keybind_binding = "";
            }
        } 

        let h = ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
            ui.label(format!("{}: ", K::display_name()));
        }).response.rect.height();
        ui.allocate_ui_with_layout(egui::vec2(160.0, h) , egui::Layout::top_down(egui::Align::LEFT), |ui| {
            let button_label = if K::display_name() == self.keybind_binding {
                "(Press a key)".to_owned()
            } else {
                ui.ctx().format_shortcut(&systems.prefs.get::<K>())
            }; 
            if ui.small_button(button_label).clicked() {
                self.keybind_binding = K::display_name();
            }
        });
        
        ui.end_row();
    }

}

impl Dialog for PrefsDialog {

    fn render(&mut self, ui: &mut egui::Ui, _state: &mut EditorState, systems: &mut EditorSystems) -> bool {
        ui.vertical_centered(|ui| {
            ui.heading("Keybinds");
        });

        let key_down = ui.input(|i| i.keys_down.clone()).iter().nth(0).map(|key| *key);
        egui::Grid::new(ui.next_auto_id()).min_col_width(120.0).show(ui, |ui| {
            self.render_keybind_setting::<DeleteKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<UndoKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<RedoKeybind>(ui, systems, &key_down);

            self.render_keybind_setting::<SelectToolKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<PencilToolKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<BucketToolKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<ColorPickerToolKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<LineToolKeybind>(ui, systems, &key_down);

            self.render_keybind_setting::<PlayKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<NewFrameKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<StepBackKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<StepForwardKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<PrevFrameKeybind>(ui, systems, &key_down);
            self.render_keybind_setting::<NextFrameKeybind>(ui, systems, &key_down);

            self.render_keybind_setting::<CenterSceneKeybind>(ui, systems, &key_down);
        });
        false
    }

    fn title(&self, _state: &crate::editor::state::EditorState) -> String {
        "Preferences".to_owned()
    }

    fn unique_dialog() -> bool {
        true
    }

}