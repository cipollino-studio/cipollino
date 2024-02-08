
use crate::{editor::EditorState, project::{action::Action, graphic::Graphic, obj::{asset::Asset, ObjPtr}}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetsPanel {
    #[serde(skip)]
    create_graphic_data: Graphic,
    #[serde(skip)]
    create_graphic_dialog_open: bool,
    #[serde(skip)]
    gfx_editing_name: ObjPtr<Graphic>,
    #[serde(skip)]
    gfx_edit_curr_name: String
}

impl AssetsPanel {

    pub fn new() -> Self {

        Self {
            create_graphic_data: Graphic::default(),
            create_graphic_dialog_open: false,
            gfx_editing_name: ObjPtr::null(),
            gfx_edit_curr_name: "".to_owned()
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if ui.button("Add Graphic").clicked() {
            self.create_graphic_dialog_open = true;
        }

        let mut close_create_graphic_dialog = false;
        let root_folder = state.project.root_folder.make_ptr();
        egui::Window::new("New Graphic")
            .collapsible(false)
            .open(&mut self.create_graphic_dialog_open)
            .show(ui.ctx(), |ui| {
                graphic_data_editor(ui, &mut self.create_graphic_data);
                if ui.button("Create").clicked() {
                    if let Some((_, acts)) = Graphic::asset_add(&mut state.project, root_folder, Graphic {
                        layers: Vec::new(),
                        name: self.create_graphic_data.name.clone(),
                        ..self.create_graphic_data
                    }) {
                        state.actions.add(Action::from_list(acts));
                    }
                    close_create_graphic_dialog = true;
                }
        });
        if close_create_graphic_dialog {
            self.create_graphic_dialog_open = false;
        }

        let mut delete_gfx = None;
        let mut rename_gfx = None;
        for gfx in state.project.root_folder.get(&state.project).graphics.iter() {
            if gfx.make_ptr() != self.gfx_editing_name {
                let label_response = ui.add(
                    egui::Label::new(gfx.get(&state.project).name.as_str())
                    .sense(egui::Sense::click()));
                if label_response.double_clicked() {
                    state.open_graphic = gfx.make_ptr(); 
                    if gfx.get(&state.project).layers.len() > 0 {
                        state.active_layer = gfx.get(&state.project).layers[0].make_ptr();
                    }
                }
                label_response.context_menu(|ui| {
                    if ui.button("Rename").clicked() {
                        self.gfx_editing_name = gfx.make_ptr();
                        self.gfx_edit_curr_name = gfx.get(&state.project).name().clone();
                        ui.close_menu();
                    }
                    if ui.button("Delete").clicked() {
                        delete_gfx = Some(gfx.make_ptr());
                        ui.close_menu();
                    }
                });
            } else {
                if ui.text_edit_singleline(&mut self.gfx_edit_curr_name).lost_focus() {
                    rename_gfx = Some(self.gfx_edit_curr_name.clone());
                }
            }
        }
        if let Some(name) = rename_gfx {
            if let Some(act) = Graphic::rename(&mut state.project, self.gfx_editing_name, name) {
                state.actions.add(Action::from_single(act));
            }
            self.gfx_editing_name = ObjPtr::null();
        }
        if let Some(gfx) = delete_gfx {
            if gfx == state.open_graphic {
                state.open_graphic = ObjPtr::null();
            }
            if let Some(acts) = Graphic::asset_delete(&mut state.project, gfx) {
                state.actions.add(Action::from_list(acts));
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
