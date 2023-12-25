

use crate::{editor::EditorState, project::{action::Action, graphic::GraphicData}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetsPanel {
    create_graphic_data: GraphicData,
    #[serde(skip)]
    create_graphic_dialog_open: bool,
}

impl AssetsPanel {

    pub fn new() -> Self {
        Self {
            create_graphic_data: GraphicData {
                name: "Graphic".to_owned(),
                len: 100, 
                clip: false,
                w: 1920,
                h: 1080
            },
            create_graphic_dialog_open: false,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if ui.button("Add Graphic").clicked() {
            self.create_graphic_dialog_open = true;
        }

        let mut close_create_graphic_dialog = false;
        egui::Window::new("New Graphic")
            .collapsible(false)
            .open(&mut self.create_graphic_dialog_open)
            .show(ui.ctx(), |ui| {
                graphic_data_editor(ui, &mut self.create_graphic_data);
                if ui.button("Create").clicked() {
                    let (key, act) = state.project.add_graphic(self.create_graphic_data.clone());
                    state.open_graphic = Some(key);
                    state.actions.add(Action::from_single(act));
                    close_create_graphic_dialog = true; 
                } 
        });
        if close_create_graphic_dialog {
            self.create_graphic_dialog_open = false;
        }

        for (key, gfx) in state.project.graphics.iter() {
            let label_response = ui.add(
                egui::Label::new(gfx.data.name.as_str())
                .sense(egui::Sense::click()));
            if label_response.double_clicked() {
                state.open_graphic = Some(*key); 
                if gfx.layers.len() > 0 {
                    state.active_layer = gfx.layers[0];
                }
            }
        }
    }

}

pub fn graphic_data_editor(ui: &mut egui::Ui, data: &mut GraphicData) {
    ui.text_edit_singleline(&mut data.name);
    ui.add(egui::DragValue::new(&mut data.len).clamp_range(1..=100000));
    ui.checkbox(&mut data.clip, "Clip");
    if data.clip {
        ui.add(egui::DragValue::new(&mut data.w).clamp_range(1..=100000));
        ui.add(egui::DragValue::new(&mut data.h).clamp_range(1..=100000));
    }
}
