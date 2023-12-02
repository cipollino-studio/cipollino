

use crate::{editor::EditorState, project::action::Action};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetsPanel {

}

impl AssetsPanel {

    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if ui.button("Add Graphic").clicked() {
            state.actions.add(Action::from_single(state.project.add_graphic("Graphic".to_owned(), 100)));
        }
        for (key, gfx) in state.project.graphics.iter() {
            let label_response = ui.add(
                egui::Label::new(gfx.data.name.as_str())
                .sense(egui::Sense::click()));
            if label_response.double_clicked() {
                state.open_graphic = Some(*key); 
            }
        }
    }

}
