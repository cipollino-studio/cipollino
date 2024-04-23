
use std::cmp;

use crate::{editor::{state::EditorState, EditorSystems}, project::{action::Action, file::{audio::AudioFile, FilePtr, FileType}, folder::Folder, graphic::Graphic, obj::{asset::Asset, obj_list::ObjListTrait, ObjBox, ObjPtr}, palette::Palette, Project}, util::ui::dnd::{dnd_drop_zone_reset_colors, dnd_drop_zone_setup_colors, draggable_label, draggable_widget}};
use crate::project::AssetPtr;

use self::graphic_dialogs::{GraphicPropertiesDialog, NewGraphicDialog};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetsPanel {
    #[serde(skip)]
    editing_name: Option<(AssetPtr, String)>
}

mod graphic_dialogs;

impl AssetsPanel {

    pub fn new() -> Self {
        Self {
            editing_name: None 
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) {
        let mut create_folder = false;
        egui::TopBottomPanel::top(ui.next_auto_id()).show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button(Graphic::icon()).clicked() {
                    systems.dialog.open_dialog(NewGraphicDialog::new());
                }
                if ui.button(Palette::icon()).clicked() {
                    let root_folder = state.project.root_folder.make_ptr();
                    if let Some((_ptr, act)) = Palette::asset_add(&mut state.project, root_folder, Palette::new(root_folder)) {
                        state.actions.add(Action::from_list(act));
                    }
                }
                if ui.button(Folder::icon()).clicked() {
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
                self.render_asset_hiearchy(ui, state, systems);
            });
        });

    }

    fn render_asset_hiearchy(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) {
        let mut open = None;
        let mut delete = None;
        let mut rename = false;
        let mut asset_transfer = None;

        let colors = dnd_drop_zone_setup_colors(ui);

        self.render_folder_contents(ui, state, systems, state.project.root_folder.make_ptr(), &mut open, &mut delete, &mut rename, &mut asset_transfer);
        let (_, root_payload) = ui.dnd_drop_zone::<(AssetPtr, ObjPtr<Folder>)>(egui::Frame::default(), |ui| {
            ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
        });
        if let Some(root_payload) = root_payload {
            let (asset, from_folder) = root_payload.as_ref().clone();
            asset_transfer = Some((state.project.root_folder.make_ptr(), asset, from_folder));
        }

        dnd_drop_zone_reset_colors(ui, colors);

        if let Some(asset) = open {
            match asset {
                AssetPtr::Graphic(gfx_ptr) => {
                    state.project.load_asset_with_key(gfx_ptr, systems.toasts);
                    if let Some(gfx) = state.project.graphics.get(gfx_ptr) { 
                        state.open_graphic = gfx_ptr; 
                        state.selection.clear();
                        if gfx.layers.len() > 0 {
                            state.active_layer = gfx.layers[0].make_ptr();
                        }
                    }
                },
                AssetPtr::Palette(palette_ptr) => {
                    state.project.load_asset_with_key(palette_ptr, systems.toasts);
                    if let Some(_) = state.project.palettes.get(palette_ptr) {
                        state.open_palette = palette_ptr;
                    }
                },
                _ => {}
            }
        }

        if let Some(asset) = delete {
            if let Some(acts) = match asset {
                AssetPtr::Folder(folder) => Folder::asset_delete(&mut state.project, folder),
                AssetPtr::Graphic(graphic) => Graphic::asset_delete(&mut state.project, graphic),
                AssetPtr::Palette(palette) => Palette::asset_delete(&mut state.project, palette),
                AssetPtr::Audio(audio) => AudioFile::delete(&mut state.project, audio).map(|act| vec![act]),
            } {
                state.actions.add(Action::from_list(acts));
            }
        }

        if rename {
            let (asset, name) = std::mem::replace(&mut self.editing_name, None).unwrap();
            if !name.is_empty() {
                if let Some(act) = match asset {
                    AssetPtr::Folder(folder) => Folder::rename(&mut state.project, folder, name),
                    AssetPtr::Graphic(graphic) => Graphic::rename(&mut state.project, graphic, name),
                    AssetPtr::Palette(palette) => Palette::rename(&mut state.project, palette, name),
                    AssetPtr::Audio(audio) => AudioFile::rename(&mut state.project, &audio, name),
                } {
                    state.actions.add(Action::from_single(act));
                }
            }
        }

        if let Some((folder, asset, from_folder)) = asset_transfer {
            if let Some(acts) = match asset {
                AssetPtr::Graphic(gfx) => {
                    state.project.load_asset_with_key(gfx, systems.toasts);
                    Graphic::asset_transfer(&mut state.project, gfx, folder) 
                },
                AssetPtr::Palette(palette) => {
                    state.project.load_asset_with_key(palette, systems.toasts);
                    Palette::asset_transfer(&mut state.project, palette, folder)
                },
                AssetPtr::Folder(subfolder) => {
                    Folder::asset_transfer(&mut state.project, subfolder, folder) 
                },
                AssetPtr::Audio(audio) => {
                    AudioFile::transfer(&mut state.project, &audio, from_folder, folder).map(|act| vec![act])
                }
            } {
                state.actions.add(Action::from_list(acts));
            } 
        }
    }

    fn begin_rename(&mut self, asset: AssetPtr, name: String) {
        self.editing_name = Some((asset, name.clone()));
    }

    fn render_asset<T, F>(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems, obj: ObjPtr<T>, folder: ObjPtr<Folder>, open: &mut Option<AssetPtr>, delete: &mut Option<AssetPtr>, rename: &mut bool, context_menu: F) where T: Asset, F: FnOnce(&mut egui::Ui) {
        let name = if let Some(name) = T::get_list(&state.project).get_name(obj) {
            name
        } else {
            return;
        };

        if self.editing_name.is_none() || T::make_asset_ptr(obj) != self.editing_name.as_ref().unwrap().0 {
            let asset_text = format!("{} {}", T::icon(), name);
            let resp = draggable_label(ui, asset_text.as_str(), (T::make_asset_ptr(obj), folder));
            if resp.double_clicked() {
                *open = Some(T::make_asset_ptr(obj));
            }
            resp.context_menu(|ui| {
                context_menu(ui);
                if ui.button("Rename").clicked() {
                    state.project.load_asset_with_key(obj, systems.toasts);
                    self.begin_rename(T::make_asset_ptr(obj), name);
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    state.project.load_asset_with_key(obj, systems.toasts);
                    *delete = Some(T::make_asset_ptr(obj));
                    ui.close_menu();
                }
            });
        } else {
            let (_, name) = self.editing_name.as_mut().unwrap();
            if ui.text_edit_singleline(name).lost_focus() {
                *rename = true;
            }
        }
    }

    fn render_file_asset<T>(&mut self, ui: &mut egui::Ui, project: &Project, file_ptr: &FilePtr<T>, folder: ObjPtr<Folder>, delete: &mut Option<AssetPtr>, rename: &mut bool) -> Option<()> where T: FileType {
        if self.editing_name.is_none() || T::make_asset_ptr(file_ptr) != self.editing_name.as_ref().unwrap().0 {
            let file = file_ptr.get(project)?;
            let asset_text = format!("{} {}", T::icon(), file.name());
            let resp = draggable_label(ui, asset_text.as_str(), (T::make_asset_ptr(file_ptr), folder));
            resp.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    self.begin_rename(T::make_asset_ptr(file_ptr), file.name().to_owned());
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    *delete = Some(T::make_asset_ptr(file_ptr));
                    ui.close_menu();
                }
            });
        } else {
            let (_, name) = self.editing_name.as_mut().unwrap();
            if ui.text_edit_singleline(name).lost_focus() {
                *rename = true;
            }
        }
        Some(())
    } 

    fn sort_asset_list<'a, T: Asset>(project: &'a Project, assets: &'a Vec<ObjBox<T>>) -> Vec<&'a ObjBox<T>> {
        let mut res = assets.iter().map(|elem| elem).collect::<Vec<&'a ObjBox<T>>>();
        res.sort_by(|a, b| {
            a.get_name(&project).to_lowercase().cmp(&b.get_name(&project).to_lowercase())
        });
        res
    }

    fn sort_file_asset_list<'a, T: FileType>(project: &'a Project, file_assets: &'a Vec<FilePtr<T>>) -> Vec<&'a FilePtr<T>> {
        let mut res = file_assets.iter().map(|elem| elem).collect::<Vec<&'a FilePtr<T>>>();
        res.sort_by(|a_ptr, b_ptr| {
            if let Some(a) = a_ptr.get(project) {
                if let Some(b) = b_ptr.get(project) {
                    return a.name().to_lowercase().cmp(&b.name().to_lowercase());
                }
            }
            cmp::Ordering::Equal 
        });
        res
    }

    fn render_folder_contents(
            &mut self,
            ui: &mut egui::Ui,
            state: &mut EditorState,
            systems: &mut EditorSystems,
            folder_ptr: ObjPtr<Folder>,
            open: &mut Option<AssetPtr>,
            delete: &mut Option<AssetPtr>,
            rename: &mut bool,
            asset_transfer: &mut Option<(ObjPtr<Folder>, AssetPtr, ObjPtr<Folder>)>) -> Option<bool> {
        let folder = state.project.folders.get(folder_ptr)?;

        let mut inner_hovered = false;
        for subfolder in Self::sort_asset_list(&state.project, &folder.folders).iter().map(|obj_box| obj_box.make_ptr()).collect::<Vec<ObjPtr<Folder>>>() {
            inner_hovered |= self.render_subfolder(ui, state, systems, folder_ptr, subfolder, open, delete, rename, asset_transfer)?;
        }

        let folder = state.project.folders.get(folder_ptr)?;
        for gfx in Self::sort_asset_list(&state.project, &folder.graphics).iter().map(|obj_box| obj_box.make_ptr()).collect::<Vec<ObjPtr<Graphic>>>() {
            let mut open_properties = false;
            let mut load = false;
            self.render_asset(ui, state, systems, gfx, folder_ptr, open, delete, rename, |ui| {
                if ui.button("Properties").clicked() {
                    load = true;
                    open_properties = true;
                    ui.close_menu();
                }
            }); 

            if load {
                state.project.load_asset_with_key(gfx, systems.toasts);
            }
            if open_properties {
                systems.dialog.open_dialog(GraphicPropertiesDialog::new(gfx)); 
            }
        }

        let folder = state.project.folders.get(folder_ptr)?;
        for palette in Self::sort_asset_list(&state.project, &folder.palettes).iter().map(|obj_box| obj_box.make_ptr()).collect::<Vec<ObjPtr<Palette>>>() {
            self.render_asset(ui, state, systems, palette, folder_ptr, open, delete, rename, |_| {});
        }

        let folder = state.project.folders.get(folder_ptr)?;
        for audio in Self::sort_file_asset_list(&state.project, &folder.audios) { 
            self.render_file_asset(ui, &state.project, audio, folder_ptr, delete, rename);
        }
        Some(inner_hovered)
    }

    fn render_subfolder(&mut self,
        ui: &mut egui::Ui,
        state: &mut EditorState,
        systems: &mut EditorSystems,
        superfolder: ObjPtr<Folder>,
        folder: ObjPtr<Folder>,
        open: &mut Option<AssetPtr>,
        delete: &mut Option<AssetPtr>,
        rename: &mut bool, 
        asset_transfer: &mut Option<(ObjPtr<Folder>, AssetPtr, ObjPtr<Folder>)>) -> Option<bool> {

        if !self.editing_name.is_none() && self.editing_name.as_ref().unwrap().0 == Folder::make_asset_ptr(folder) {
            if ui.text_edit_singleline(&mut self.editing_name.as_mut().unwrap().1).lost_focus() {
                *rename = true;
            }
            return Some(false);
        }

        let is_anything_being_dragged = egui::DragAndDrop::has_any_payload(ui.ctx());
        let can_accept_what_is_being_dragged = egui::DragAndDrop::has_payload_of_type::<(AssetPtr, ObjPtr<Folder>)>(ui.ctx());

        let mut frame = egui::Frame::default().begin(ui);
        let mut inner_hovered = false;
        let folder_resp = draggable_widget(&mut frame.content_ui, (AssetPtr::Folder(folder), superfolder), |ui, _| {
            let name = state.project.folders.get(folder).unwrap().name.to_owned();
            let resp = ui.collapsing(name, |ui| {
                inner_hovered |= self.render_folder_contents(ui, state, systems, folder, open, delete, rename, asset_transfer).unwrap_or(false);
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
                self.begin_rename(Folder::make_asset_ptr(folder), state.project.folders.get(folder).unwrap().name.clone());
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                *delete = Some(Folder::make_asset_ptr(folder)); 
                ui.close_menu();
            }
        });

        if !inner_hovered {
            if let Some(payload) = response.dnd_release_payload::<(AssetPtr, ObjPtr<Folder>)>() {
                let (asset, from_folder) = payload.as_ref().clone();
                *asset_transfer = Some((folder, asset, from_folder));
            }
        }

        Some(hovered || inner_hovered)
    }

}
