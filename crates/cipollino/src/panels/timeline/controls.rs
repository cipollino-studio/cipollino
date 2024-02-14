
use crate::{editor::EditorState, project::{action::Action, graphic::Graphic, layer::Layer, obj::child_obj::ChildObj}};

use super::{next_keyframe, prev_keyframe, TimelinePanel};

pub fn timeline_controls(timeline: &mut TimelinePanel, ui: &mut egui::Ui, state: &mut EditorState) {

    if ui.button("+").clicked() {
        if let Some((layer, act)) = Layer::add(&mut state.project, state.open_graphic, Layer {
            graphic: state.open_graphic,
            name: "Layer".to_owned(),
            frames: Vec::new()
        }) {
            state.actions.add(Action::from_single(act));
            state.active_layer = layer;
        }
    }

    if ui.button("<<").clicked() {
        state.time = 0.0;
        state.playing = false;
    }
    if ui.button("*<").clicked() {
        prev_keyframe(state);
        state.playing = false;
    }
    if ui.button(if state.playing { "||" } else { ">" }).clicked() {
        state.playing = !state.playing; 
    }
    if ui.button(">*").clicked() {
        next_keyframe(state);
        state.playing = false;
    }

    let gfx = state.project.graphics.get(state.open_graphic).unwrap();
    if ui.button(">>").clicked() {
        state.time = (gfx.len as f32 - 1.0) * state.frame_len();
        state.playing = false;
    }

    let mut len = gfx.len; 
    ui.label("Graphic length: ");
    let gfx_len_drag = ui.add(egui::DragValue::new(&mut len).clamp_range(1..=1000000).update_while_editing(false));
    let len_changed = len != gfx.len;
    if len_changed {
        if let Some(act) = Graphic::set_len(&mut state.project, state.open_graphic, len) {
            state.actions.add(Action::from_single(act));
        } 
    }
    if gfx_len_drag.drag_released() || (!gfx_len_drag.dragged() && len_changed) {
        state.actions.add(std::mem::replace(&mut timeline.set_gfx_len_action, Action::new()));
    }

    ui.label("Onion skin:");
    ui.add(egui::DragValue::new(&mut state.onion_before).clamp_range(0..=10));
    ui.add(egui::DragValue::new(&mut state.onion_after).clamp_range(0..=10));

}
