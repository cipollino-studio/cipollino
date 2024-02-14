
use egui::Vec2;

use crate::{editor::EditorState, project::{action::Action, layer::Layer, obj::{child_obj::ChildObj, ObjPtr}}};

use super::{FrameGridRow, TimelinePanel};

impl FrameGridRow {

    fn render_layer_layer(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, rect: egui::Rect, state: &mut EditorState, layer_ptr: ObjPtr<Layer>) -> Option<()> {
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
            let label = egui::Label::new(layer.name.clone()).sense(egui::Sense::click()).selectable(false);
            let (_, galley, _) = label.layout_in_ui(ui);
            let text_rect = galley.rect;
            let label = egui::Label::new(layer.name.clone()).sense(egui::Sense::click()).selectable(false);
            let mut text_put_rect = rect.clone();
            text_put_rect.set_width(text_rect.width());
            let layer_name_response = ui.put(text_put_rect, label);
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
    
        None
    }

    fn render_layer(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, rect: egui::Rect, state: &mut EditorState) {
        match &self.kind {
            super::FrameGridRowKind::Layer(layer) => self.render_layer_layer(timeline, ui, rect, state, *layer),
        };
    }

}

pub fn layers(timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_h: f32, state: &mut EditorState, grid_rows: &Vec<FrameGridRow>) {
    let mut i = 0;
    let (rect, _response) = ui.allocate_exact_size(Vec2::new(100.0, (grid_rows.len() as f32) * frame_h), egui::Sense::click());
    let tl = rect.left_top(); 
    for row in grid_rows {
        let layer_name_tl = tl + Vec2::new(0.0, frame_h * (i as f32)); 
        let layer_name_br = layer_name_tl + Vec2::new(100.0, frame_h); 
        let rect = egui::Rect::from_min_max(layer_name_tl, layer_name_br);
        row.render_layer(timeline, ui, rect, state);
        i += 1;
    }
   
}
