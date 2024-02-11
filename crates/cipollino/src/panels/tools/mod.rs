
use std::sync::Arc;

use glam::Vec2;

use crate::{editor::EditorState, project::{action::ObjAction, frame::Frame, layer::Layer, obj::{child_obj::ChildObj, ObjPtr}, Project}};

use super::scene::{OverlayRenderer, ScenePanel};

pub mod pencil;
pub mod select;
pub mod bucket;

pub trait Tool {

    fn mouse_click(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {}
    fn mouse_down(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _scene: &mut ScenePanel) {}
    fn mouse_release(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {}
    fn mouse_cursor(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> egui::CursorIcon {
        egui::CursorIcon::Default
    }
    fn draw_overlay(&mut self, _overlay: &mut OverlayRenderer, _state: &EditorState) {}
    fn tool_panel(&mut self, _ui: &mut egui::Ui, _state: &mut EditorState) {}
    fn reset(&mut self, _state: &mut EditorState) {}

    fn get_icon(&self) -> &str;
    fn name(&self) -> &str;
    fn shortcut(&self) -> egui::KeyboardShortcut;

}

pub fn active_frame_proj_layer_frame(project: &mut Project, active_layer: ObjPtr<Layer>, frame: i32) -> Option<(ObjPtr<Frame>, Option<ObjAction>)> {
    let layer = project.layers.get(active_layer)?;
    if let Some(frame) = layer.get_frame_exactly_at(project, frame) {
        Some((frame.make_ptr(), None))
    } else {
        let (frame, act) = Frame::add(project, active_layer, Frame {
            layer: active_layer,
            time: frame,
            strokes: Vec::new()
        }).unwrap();
        Some((frame, Some(act)))
    }
}

pub fn active_frame(state: &mut EditorState) -> Option<(ObjPtr<Frame>, Option<ObjAction>)> {
    let frame = state.frame();
    active_frame_proj_layer_frame(&mut state.project, state.active_layer, frame)
}