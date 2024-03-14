
use crate::{editor::state::EditorState, project::{action::Action, graphic::Graphic, layer::{Layer, LayerKind}, obj::child_obj::ChildObj}};

use super::{next_keyframe, prev_keyframe, TimelinePanel};

pub fn timeline_controls(timeline: &mut TimelinePanel, ui: &mut egui::Ui, state: &mut EditorState) {

    if ui.button(egui_phosphor::regular::FILE_PLUS).clicked() {
        if let Some((layer, act)) = Layer::add(&mut state.project, state.open_graphic, Layer {
            graphic: state.open_graphic,
            name: "Layer".to_owned(),
            show: true,
            kind: LayerKind::Animation, 
            frames: Vec::new(),
            sound_instances: Vec::new()
        }) {
            state.actions.add(Action::from_single(act));
            state.active_layer = layer;
        }
    }

    if ui.button(egui_phosphor::regular::REWIND).clicked() {
        state.time = 0;
        state.pause();
    }
    if ui.button(egui_phosphor::regular::CARET_CIRCLE_LEFT).clicked() {
        prev_keyframe(state);
        state.pause();
    }
    if ui.button(if state.playing { egui_phosphor::regular::PAUSE } else { egui_phosphor::regular::PLAY }).clicked() {
        if state.playing {
            state.pause();
        } else {
            state.play();
        }
    }
    if ui.button(egui_phosphor::regular::CARET_CIRCLE_RIGHT).clicked() {
        next_keyframe(state);
        state.pause();
    }

    let gfx = state.project.graphics.get(state.open_graphic).unwrap();
    if ui.button(egui_phosphor::regular::FAST_FORWARD).clicked() {
        state.time = ((gfx.len as f32 - 1.0) * state.frame_len() / state.sample_len()).floor() as i64;
        state.pause();
    }

    let gfx = state.project.graphics.get(state.open_graphic).unwrap();

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
