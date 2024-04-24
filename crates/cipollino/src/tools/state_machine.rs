
use std::sync::Arc;

use glam::Vec2;

use crate::{editor::state::EditorState, panels::scene::{overlay::OverlayRenderer, ScenePanel}};


pub trait ToolState: Send + Sync {

    fn mouse_click(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> Option<Box<dyn ToolState>> { None }
    fn mouse_down(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _scene: &mut ScenePanel) -> Option<Box<dyn ToolState>> { None }
    fn mouse_release(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> Option<Box<dyn ToolState>> { None }
    fn mouse_cursor(&mut self, _mouse_pos: Vec2, _state: &mut EditorState, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> egui::CursorIcon {
        egui::CursorIcon::Default
    }
    fn draw_overlay(&mut self, _overlay: &mut OverlayRenderer, _state: &EditorState) {}

}

pub struct ToolStateMachine {
    state: Box<dyn ToolState>
}

impl ToolStateMachine {

    pub fn new<T: ToolState>(init_state: T) -> Self where T: 'static {
        ToolStateMachine {
            state: Box::new(init_state)
        }
    }

    pub fn mouse_click(&mut self, mouse_pos: Vec2, state: &mut EditorState, ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        if let Some(new_state) = self.state.mouse_click(mouse_pos, state, ui, scene, gl) {
            self.state = new_state;
        }
    }

    pub fn mouse_down(&mut self, mouse_pos: Vec2, state: &mut EditorState, scene: &mut ScenePanel) {
        if let Some(new_state) = self.state.mouse_down(mouse_pos, state, scene) {
            self.state = new_state;
        }
    }

    pub fn mouse_release(&mut self, mouse_pos: Vec2, state: &mut EditorState, ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        if let Some(new_state) = self.state.mouse_release(mouse_pos, state, ui, scene, gl) {
            self.state = new_state;
        }
    }

    pub fn mouse_cursor(&mut self, mouse_pos: Vec2, state: &mut EditorState, scene: &mut ScenePanel, gl: &Arc<glow::Context>) -> egui::CursorIcon {
        self.state.mouse_cursor(mouse_pos, state, scene, gl)
    }

    pub fn draw_overlay(&mut self, overlay: &mut OverlayRenderer, state: &EditorState) {
        self.state.draw_overlay(overlay, state);
    }

}

#[macro_export]
macro_rules! use_tool_state_machine {
    ($name: ident) => {

        fn mouse_click(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::state::EditorState, ui: &mut egui::Ui, scene: &mut crate::panels::scene::ScenePanel, gl: &std::sync::Arc<glow::Context>) {
            self.$name.mouse_click(mouse_pos, state, ui, scene, gl);
        }

        fn mouse_down(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::state::EditorState, scene: &mut crate::panels::scene::ScenePanel) {
            self.$name.mouse_down(mouse_pos, state, scene);
        }

        fn mouse_release(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::state::EditorState, ui: &mut egui::Ui, scene: &mut crate::panels::scene::ScenePanel, gl: &std::sync::Arc<glow::Context>) {
            self.$name.mouse_release(mouse_pos, state, ui, scene, gl);
        }

        fn mouse_cursor(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::state::EditorState, scene: &mut crate::panels::scene::ScenePanel, gl: &std::sync::Arc<glow::Context>) -> egui::CursorIcon {
            self.$name.mouse_cursor(mouse_pos, state, scene, gl)
        }

        fn draw_overlay(&mut self, overlay: &mut crate::panels::scene::overlay::OverlayRenderer, state: &crate::editor::state::EditorState) {
            self.$name.draw_overlay(overlay, state);
        }

    };
}
