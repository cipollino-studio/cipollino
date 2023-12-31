use std::sync::Arc;

use glam::Vec2;

use crate::{editor::EditorState, project::action::ObjAction};

use super::scene::{OverlayRenderer, ScenePanel};

pub mod pencil;
pub mod select;

pub trait Tool {

    fn mouse_click(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {}
    fn mouse_down(&mut self, _mouse_pos: Vec2, _state: &mut EditorState) {}
    fn mouse_release(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _ui: &mut egui::Ui) {}
    fn mouse_cursor(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> egui::CursorIcon {
        egui::CursorIcon::Default
    }
    fn draw_overlay(&mut self, _overlay: &mut OverlayRenderer, _state: &mut EditorState) {}
    fn tool_panel(&mut self, _ui: &mut egui::Ui, _state: &mut EditorState) {}
    fn reset(&mut self, _state: &mut EditorState) {}

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
