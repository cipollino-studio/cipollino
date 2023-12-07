use glam::Vec2;

use crate::{editor::EditorState, project::action::ObjAction};

pub mod pencil;

pub trait Tool {

    fn mouse_click(&mut self, _mouse_pos: Vec2, _state: &mut EditorState) {}
    fn mouse_down(&mut self, _mouse_pos: Vec2, _state: &mut EditorState) {}
    fn mouse_release(&mut self, _mouse_pos: Vec2, _state: &mut EditorState) {}

}

pub fn active_frame(state: &mut EditorState) -> (u64, Option<ObjAction>) {
    if let Some(frame) = state.project.get_frame_exactly_at(state.active_layer, state.frame()) {
        (frame, None) 
    } else {
        if let Some((frame, act)) = state.project.add_frame(state.active_layer, state.frame()) {
            (frame, Some(act))
        } else {
            (0, None)
        }
    }
} 
