

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
            let (key, act) = state.project.add_graphic("Graphic".to_owned(), 100);
            let (_layer_key, layer_act) = state.project.add_layer(key, "Layer".to_owned()).unwrap();
            state.actions.add(Action::from_list(vec![act, layer_act]));
        }
        for (key, gfx) in state.project.graphics.iter() {
            let label_response = ui.add(
                egui::Label::new(gfx.data.name.as_str())
                .sense(egui::Sense::click()));
            if label_response.double_clicked() {
                state.open_graphic = Some(*key); 
                state.active_layer = gfx.layers[0];
            }
        }
    }

}
