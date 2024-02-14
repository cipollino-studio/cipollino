
use egui::Vec2;

use crate::{editor::EditorState, project::{action::Action, layer::Layer, obj::{child_obj::ChildObj, ObjPtr}}, util::ui::{dnd_drop_zone_reset_colors, dnd_drop_zone_setup_colors, draggable_widget, label_size}};

use super::{FrameGridRow, TimelinePanel};

impl FrameGridRow {

    fn render_layer_layer(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, rect: egui::Rect, state: &mut EditorState, response: &egui::Response, layer_drop_idx: &mut Option<usize>, layer_ptr: ObjPtr<Layer>) -> Option<()> {
        let layer = state.project.layers.get(layer_ptr)?; 

        let mut set_name = false;
        let mut delete_layer = false;

        if layer_ptr == state.active_layer {
            ui.painter().rect(rect, 0.0, super::HIGHLIGHT, egui::Stroke::NONE);
        }
        if timeline.layer_editing_name == layer_ptr {
            let text_input = egui::TextEdit::singleline(&mut timeline.layer_edit_curr_name);
            let response = ui.put(rect, text_input);
            if response.lost_focus() {
                set_name = true;
            }
        } else {
            let name_size = label_size(ui, egui::Label::new(layer.name.clone()).selectable(false));
            let mut text_rect = rect.clone();
            text_rect.set_width(name_size.x);
            let layer_name_response = draggable_widget(ui, layer_ptr, move |ui| {
                let mut label = egui::Label::new(layer.name.clone()).selectable(false);
                if !egui::DragAndDrop::has_payload_of_type::<ObjPtr<Layer>>(ui.ctx()) {
                    label = label.sense(egui::Sense::click());
                }
                let layer_name_response = ui.put(text_rect, label);
                (layer_name_response.clone(), layer_name_response)
            });
            layer_name_response.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    timeline.layer_editing_name = layer_ptr;
                    timeline.layer_edit_curr_name = layer.name.clone();
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    delete_layer = true;
                    ui.close_menu();
                }
            });
            if layer_name_response.clicked() {
                state.active_layer = layer_ptr;
            }
        }
    
        if delete_layer {
            if let Some(act) = Layer::delete(&mut state.project, layer_ptr) {
                state.actions.add(Action::from_single(act));
            }
        }
        if set_name {
            if let Some(act) = Layer::set_name(&mut state.project, timeline.layer_editing_name, timeline.layer_edit_curr_name.clone()) {
                state.actions.add(Action::from_single(act));
            }
            timeline.layer_editing_name = ObjPtr::null();
        }

        if let (Some(pointer), Some(_)) = (
            ui.input(|i| i.pointer.hover_pos()),
            response.dnd_hover_payload::<ObjPtr<Layer>>(),
        ) {
            if rect.contains(pointer) {
                let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                if pointer.y < rect.center().y {
                    ui.painter().hline(rect.x_range(), rect.top(), stroke);
                    *layer_drop_idx = Some(self.idx);
                } else {
                    ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                    *layer_drop_idx = Some(self.idx + 1);
                }
            }
        }
    
        None
    }

    fn render_layer(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, rect: egui::Rect, state: &mut EditorState, response: &egui::Response, layer_drop_idx: &mut Option<usize>) {
        match &self.kind {
            super::FrameGridRowKind::Layer(layer) => self.render_layer_layer(timeline, ui, rect, state, response, layer_drop_idx, *layer),
        };
    }

}

pub fn layers(timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_h: f32, state: &mut EditorState, grid_rows: &Vec<FrameGridRow>) {
    let mut i = 0;
    let colors = dnd_drop_zone_setup_colors(ui);
    let init_stroke = std::mem::replace(&mut ui.visuals_mut().widgets.active.bg_stroke.color, egui::Color32::TRANSPARENT);
    let mut layer_drop_idx = None;
    if let (_, Some(payload)) = ui.dnd_drop_zone::<ObjPtr<Layer>>(egui::Frame::default(), |ui| {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(100.0, (grid_rows.len() as f32) * frame_h), egui::Sense::click());
        let tl = rect.left_top(); 
        for row in grid_rows {
            let layer_name_tl = tl + Vec2::new(0.0, frame_h * (i as f32)); 
            let layer_name_br = layer_name_tl + Vec2::new(100.0, frame_h); 
            let rect = egui::Rect::from_min_max(layer_name_tl, layer_name_br);
            row.render_layer(timeline, ui, rect, state, &response, &mut layer_drop_idx);
            i += 1;
        }
    }) {
        if let Some(new_idx) = layer_drop_idx {
            let layer = *payload.as_ref();
            if let Some(act) = Layer::set_index(&mut state.project, layer, new_idx) {
                state.actions.add(Action::from_single(act));
            }
        }
    }
    dnd_drop_zone_reset_colors(ui, colors);
    ui.visuals_mut().widgets.active.bg_stroke.color = init_stroke;
}
