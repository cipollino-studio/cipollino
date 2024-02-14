
use crate::{editor::EditorState, project::{action::Action, folder::Folder, graphic::Graphic, obj::{asset::Asset, ObjBox, ObjPtr}}, util::ui::{dnd_drop_zone_reset_colors, dnd_drop_zone_setup_colors, draggable_label, draggable_widget}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetsPanel {
    #[serde(skip)]
    create_graphic_data: Graphic,
    #[serde(skip)]
    create_graphic_dialog_open: bool,

    #[serde(skip)]
    gfx_editing_name: ObjPtr<Graphic>,
    #[serde(skip)]
    gfx_edit_curr_name: String,
    #[serde(skip)]
    folder_editing_name: ObjPtr<Folder>,
    #[serde(skip)]
    folder_edit_curr_name: String
}

#[derive(Clone, Copy)]
enum AssetDragPayload {
    Graphic(ObjPtr<Graphic>),
    Folder(ObjPtr<Folder>)
}

impl AssetsPanel {

    pub fn new() -> Self {

        Self {
            create_graphic_data: Graphic::default(),
            create_graphic_dialog_open: false,
            gfx_editing_name: ObjPtr::null(),
            gfx_edit_curr_name: "".to_owned(),
            folder_editing_name: ObjPtr::null(),
            folder_edit_curr_name: "".to_owned()
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        let mut create_folder = false;
        egui::TopBottomPanel::top(ui.next_auto_id()).show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button(egui_phosphor::regular::IMAGE_SQUARE).clicked() {
                    self.create_graphic_dialog_open = true;
                }
                if ui.button(egui_phosphor::regular::FOLDER).clicked() {
                    create_folder = true;
                }
            });
        });
        if create_folder {
            let root_folder = state.project.root_folder.make_ptr();
            if let Some((_ptr, acts)) = Folder::asset_add(&mut state.project, root_folder, Folder::new(root_folder)) {
                state.actions.add(Action::from_list(acts));
            }
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.render_asset_hiearchy(ui, state);
            });
        });

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
        
    }

    fn render_asset_hiearchy(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        let mut open_gfx = None;
        let mut delete_gfx = None;
        let mut delete_folder = None;
        let mut rename_gfx = None;
        let mut rename_folder = None;
        let mut asset_transfer = None;

        let colors = dnd_drop_zone_setup_colors(ui);

        self.render_folder_contents(ui, state, state.project.root_folder.make_ptr(), &mut open_gfx, &mut delete_gfx, &mut delete_folder, &mut rename_gfx, &mut rename_folder, &mut asset_transfer);
        let (_, root_payload) = ui.dnd_drop_zone::<AssetDragPayload>(egui::Frame::default(), |ui| {
            ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        });
        if let Some(root_payload) = root_payload {
            asset_transfer = Some((state.project.root_folder.make_ptr(), *root_payload.as_ref()));
        }

        dnd_drop_zone_reset_colors(ui, colors);

        if let Some(gfx_ptr) = open_gfx {
            if let Some(gfx) = state.project.graphics.get(gfx_ptr) { 
                state.open_graphic = gfx_ptr; 
                if gfx.layers.len() > 0 {
                    state.active_layer = gfx.layers[0].make_ptr();
                }
            }
        }

        if let Some(gfx) = delete_gfx {
            if gfx == state.open_graphic {
                state.open_graphic = ObjPtr::null();
            }
            if let Some(acts) = Graphic::asset_delete(&mut state.project, gfx) {
                state.actions.add(Action::from_list(acts));
            }
        }

        if let Some(folder) = delete_folder {
            if let Some(acts) = Folder::asset_delete(&mut state.project, folder) {
                state.actions.add(Action::from_list(acts));
            }
        }

        if let Some(name) = rename_gfx {
            if let Some(act) = Graphic::rename(&mut state.project, self.gfx_editing_name, name) {
                state.actions.add(Action::from_single(act));
            }
            self.gfx_editing_name = ObjPtr::null();
        }

        if let Some(name) = rename_folder {
            if let Some(act) = Folder::rename(&mut state.project, self.folder_editing_name, name) {
                state.actions.add(Action::from_single(act));
            }
            self.folder_editing_name = ObjPtr::null();
        }
        
        if let Some((folder, asset)) = asset_transfer {
            if let Some(acts) = match asset {
                AssetDragPayload::Graphic(gfx) => {
                    Graphic::asset_transfer(&mut state.project, gfx, folder) 
                },
                AssetDragPayload::Folder(subfolder) => {
                    Folder::asset_transfer(&mut state.project, subfolder, folder) 
                },
            } {
                state.actions.add(Action::from_list(acts));
            } 
        }
    }

    fn render_folder_contents(
            &mut self,
            ui: &mut egui::Ui, state: &EditorState,
            folder_ptr: ObjPtr<Folder>,
            open_gfx: &mut Option<ObjPtr<Graphic>>,
            delete_gfx: &mut Option<ObjPtr<Graphic>>,
            delete_folder: &mut Option<ObjPtr<Folder>>,
            rename_gfx: &mut Option<String>,
            rename_folder: &mut Option<String>,
            asset_transfer: &mut Option<(ObjPtr<Folder>, AssetDragPayload)>) -> Option<bool> {
        let folder = state.project.folders.get(folder_ptr)?;

        let mut inner_hovered = false;
        for subfolder in &folder.folders {
            inner_hovered |= self.render_subfolder(ui, state, subfolder, open_gfx, delete_gfx, delete_folder, rename_gfx, rename_folder, asset_transfer)?;
        }

        for gfx in &folder.graphics {
            if gfx.make_ptr() != self.gfx_editing_name {
                let gfx_text = format!("{} {}", egui_phosphor::regular::IMAGE_SQUARE, gfx.get(&state.project).name.as_str());
                let resp = draggable_label(ui, gfx_text.as_str(), AssetDragPayload::Graphic(gfx.make_ptr()));
                if resp.double_clicked() {
                    *open_gfx = Some(gfx.make_ptr());
                }
                resp.context_menu(|ui| {
                    if ui.button("Rename").clicked() {
                        self.gfx_editing_name = gfx.make_ptr();
                        self.gfx_edit_curr_name = gfx.get(&state.project).name().clone();
                        ui.close_menu();
                    }
                    if ui.button("Delete").clicked() {
                        *delete_gfx = Some(gfx.make_ptr());
                        ui.close_menu();
                    }
                });
            } else {
                if ui.text_edit_singleline(&mut self.gfx_edit_curr_name).lost_focus() {
                    *rename_gfx = Some(self.gfx_edit_curr_name.clone());
                }
            }
        }
        Some(inner_hovered)
    }

    // Adapted from ui.dnd_drop_zone
    fn render_subfolder(&mut self,
        ui: &mut egui::Ui,
        state: &EditorState,
        folder: &ObjBox<Folder>,
        open_gfx: &mut Option<ObjPtr<Graphic>>,
        delete_gfx: &mut Option<ObjPtr<Graphic>>,
        delete_folder: &mut Option<ObjPtr<Folder>>,
        rename_gfx: &mut Option<String>,
        rename_folder: &mut Option<String>,
        asset_transfer: &mut Option<(ObjPtr<Folder>, AssetDragPayload)>) -> Option<bool> {

        if self.folder_editing_name == folder.make_ptr() {
            if ui.text_edit_singleline(&mut self.folder_edit_curr_name).lost_focus() {
                *rename_folder = Some(self.folder_edit_curr_name.clone());
            }
            return Some(false);
        }

        let is_anything_being_dragged = egui::DragAndDrop::has_any_payload(ui.ctx());
        let can_accept_what_is_being_dragged = egui::DragAndDrop::has_payload_of_type::<AssetDragPayload>(ui.ctx());

        let mut frame = egui::Frame::default().begin(ui);
        let mut inner_hovered = false;
        let folder_resp = draggable_widget(&mut frame.content_ui, AssetDragPayload::Folder(folder.make_ptr()), |ui| {
            let resp = ui.collapsing(folder.get(&state.project).name.as_str(), |ui| {
                inner_hovered |= self.render_folder_contents(ui, state, folder.make_ptr(), open_gfx, delete_gfx, delete_folder, rename_gfx, rename_folder, asset_transfer).unwrap_or(false);
            }).header_response;
            (resp.clone(), resp)
        });
        let response = frame.allocate_space(ui);

        let (stroke, hovered) = if is_anything_being_dragged
            && can_accept_what_is_being_dragged
            && response.contains_pointer()
            && !inner_hovered {
            (ui.visuals().widgets.active.bg_stroke, true)
        } else {
            (ui.visuals().widgets.inactive.bg_stroke, false)
        };

        frame.frame.fill = egui::Color32::TRANSPARENT;
        frame.frame.stroke = stroke;

        frame.paint(ui);

        folder_resp.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                self.folder_editing_name = folder.make_ptr();
                self.folder_edit_curr_name = folder.get(&state.project).name.clone();
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                *delete_folder = Some(folder.make_ptr());
                ui.close_menu();
            }
        });

        if !inner_hovered {
            if let Some(payload) = response.dnd_release_payload::<AssetDragPayload>() {
                *asset_transfer = Some((folder.make_ptr(), *payload.as_ref()));
            }
        }

        Some(hovered || inner_hovered)
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
