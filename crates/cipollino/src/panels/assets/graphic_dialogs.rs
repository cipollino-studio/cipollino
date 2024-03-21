
use unique_type_id::UniqueTypeId;

use crate::{editor::{dialog::Dialog, state::EditorState}, project::{action::Action, graphic::Graphic, obj::{asset::Asset, ObjPtr}}, util::ui::drag_value};

#[derive(UniqueTypeId)]
pub struct NewGraphicDialog {
    create_graphic_data: Graphic
}

impl NewGraphicDialog {

    pub fn new() -> Self {
        Self {
            create_graphic_data: Graphic::default() 
        }
    }

}

impl Dialog for NewGraphicDialog {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool {
        ui.horizontal(|ui| {
            ui.label("Graphic name:");
            ui.text_edit_singleline(&mut self.create_graphic_data.name);
        });
        gfx_properties_ui(ui, &mut self.create_graphic_data.len, None, &mut self.create_graphic_data.clip, &mut self.create_graphic_data.w, None, &mut self.create_graphic_data.h, None);

        let mut close_dialog = false;
        let root_folder = state.project.root_folder.make_ptr();
        
        ui.vertical_centered(|ui| {
            if ui.button("Create").clicked() {
                if let Some((_, acts)) = Graphic::asset_add(&mut state.project, root_folder, Graphic {
                    layers: Vec::new(),
                    name: self.create_graphic_data.name.clone(),
                    ..self.create_graphic_data
                }) {
                    state.actions.add(Action::from_list(acts));
                }
                close_dialog = true;
            }
        });

        close_dialog 
    }

    fn title(&self, _: &EditorState) -> String {
        "New Graphic".to_owned()
    }

    fn unique_dialog() -> bool {
        true
    }

}

#[derive(UniqueTypeId)]
pub struct GraphicPropertiesDialog {
    gfx_ptr: ObjPtr<Graphic>,
    len_action: Option<Action>,
    w_action: Option<Action>,
    h_action: Option<Action>,
}

impl GraphicPropertiesDialog {

    pub fn new(gfx_ptr: ObjPtr<Graphic>) -> Self {
        Self {
            gfx_ptr,
            len_action: None,
            w_action: None,
            h_action: None 
        } 
    }

}

impl Dialog for GraphicPropertiesDialog {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool {
        let gfx = if let Some(gfx) = state.project.graphics.get_mut(self.gfx_ptr) {
            gfx
        } else {
            return true;
        };

        let mut len = gfx.len;
        let mut change_len = (false, false);
        let mut clip = gfx.clip;
        let initial_clip = clip;
        let mut w = gfx.w;
        let mut change_w = (false, false);
        let mut h = gfx.h;
        let mut change_h = (false, false);

        gfx_properties_ui(ui, &mut len, Some(&mut change_len), &mut clip, &mut w, Some(&mut change_w), &mut h, Some(&mut change_h));

        let (edit_len, set_len) = change_len;
        if edit_len {
            self.len_action.as_ref().map(|action| action.undo(&mut state.project));
            if let Some(act) = Graphic::set_len(&mut state.project, self.gfx_ptr, len) {
                let new_action = Action::from_single(act);
                if !set_len {
                    self.len_action = Some(new_action);
                } else {
                    state.actions.add(new_action);
                    self.len_action = None;
                }
            }
        }
        
        if clip != initial_clip {
            if let Some(act) = Graphic::set_clip(&mut state.project, self.gfx_ptr, clip) {
                state.actions.add(Action::from_single(act));
            }
        }

        let (edit_w, set_w) = change_w;
        if edit_w {
            self.w_action.as_ref().map(|action| action.undo(&mut state.project));
            if let Some(act) = Graphic::set_w(&mut state.project, self.gfx_ptr, w) {
                let new_action = Action::from_single(act);
                if !set_w {
                    self.w_action = Some(new_action);
                } else {
                    state.actions.add(new_action);
                    self.w_action = None;
                }
            }
        }

        let (edit_h, set_h) = change_h;
        if edit_h {
            self.h_action.as_ref().map(|action| action.undo(&mut state.project));
            if let Some(act) = Graphic::set_h(&mut state.project, self.gfx_ptr, h) {
                let new_action = Action::from_single(act);
                if !set_h {
                    self.h_action = Some(new_action);
                } else {
                    state.actions.add(new_action);
                    self.h_action = None;
                }
            }
        }

        false
    }

    fn title(&self, state: &EditorState) -> String {
        format!("{} Properties", state.project.graphics.get(self.gfx_ptr).map(|gfx| gfx.name.as_str()).unwrap_or(""))
    }

    fn unique_dialog() -> bool {
        true        
    }

}

fn gfx_properties_ui(ui: &mut egui::Ui, len: &mut u32, change_len: Option<&mut (bool, bool)>, clip: &mut bool, w: &mut u32, change_w: Option<&mut (bool, bool)>, h: &mut u32, change_h: Option<&mut (bool, bool)>) {
    drag_value(ui, "Length", len, 1..=100000, change_len);
    ui.horizontal(|ui| {
        if ui.radio(!*clip, "Graphic").clicked() {
            *clip = false; 
        }
        if ui.radio(*clip, "Clip").clicked() {
            *clip = true; 
        }
    });
    egui::Frame::default()
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::same(8.0))
        .stroke(ui.ctx().style().visuals.window_stroke)
        .show(ui, |ui| {
            ui.set_enabled(*clip);
            drag_value(ui, "Width", w, 1..=100000, change_w);
            drag_value(ui, "Height", h, 1..=100000, change_h);
            ui.set_enabled(true);
    });
}
