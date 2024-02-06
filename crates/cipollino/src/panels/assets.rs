

use std::{cell::RefCell, rc::Rc};

use crate::{editor::EditorState, project::{action::{Action, ObjAction}, graphic::Graphic, obj::asset::next_valid_name}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetsPanel {
    #[serde(skip)]
    create_graphic_data: Graphic,
    #[serde(skip)]
    create_graphic_dialog_open: bool,
}

impl AssetsPanel {

    pub fn new() -> Self {

        Self {
            create_graphic_data: Graphic::default(),
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
                    let gfx = state.project.graphics.add(Graphic {
                        layers: Vec::new(),
                        name: next_valid_name(&state.project, self.create_graphic_data.name.clone(), &state.project.root_graphics),
                        ..self.create_graphic_data
                    });
                    let gfx_ptr = gfx.make_ptr();
                    state.project.root_graphics.push(gfx);

                    let obj_store_redo = Rc::new(RefCell::new(None));
                    let obj_store_undo = obj_store_redo.clone();
                    state.actions.add(Action::from_single(ObjAction::new(move |proj| {
                        let obj = obj_store_undo.replace(None).unwrap();
                        proj.root_graphics.push(obj);
                    }, move |proj| {
                        let idx = proj.root_graphics.iter().position(|gfx| gfx.make_ptr() == gfx_ptr).unwrap();
                        let obj = proj.root_graphics.remove(idx);
                        *obj_store_redo.borrow_mut() = Some(obj);
                    })));

                    close_create_graphic_dialog = true;
                }
        });
        if close_create_graphic_dialog {
            self.create_graphic_dialog_open = false;
        }

        for gfx in state.project.root_graphics.iter() {
            let label_response = ui.add(
                egui::Label::new(gfx.get(&state.project).name.as_str())
                .sense(egui::Sense::click()));
            if label_response.double_clicked() {
                state.open_graphic = gfx.make_ptr(); 
                if gfx.get(&state.project).layers.len() > 0 {
                    state.active_layer = gfx.get(&state.project).layers[0].make_ptr();
                }
            }
        }
    }

}

pub fn graphic_data_editor(ui: &mut egui::Ui, data: &mut Graphic) {
    ui.text_edit_singleline(&mut data.name);
    ui.add(egui::DragValue::new(&mut data.len).clamp_range(1..=100000));
    ui.checkbox(&mut data.clip, "Clip");
    if data.clip {
        ui.add(egui::DragValue::new(&mut data.w).clamp_range(1..=100000));
        ui.add(egui::DragValue::new(&mut data.h).clamp_range(1..=100000));
    }
}
