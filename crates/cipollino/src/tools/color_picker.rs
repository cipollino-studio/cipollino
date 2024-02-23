
use std::sync::Arc;

use glam::Vec2;

use crate::{editor::EditorState, panels::scene::ScenePanel};

use super::Tool;



pub struct ColorPicker {

}

impl ColorPicker {

    pub fn new() -> Self {
        Self {

        }
    }

}

impl Tool for ColorPicker {

    fn mouse_click(&mut self, mouse_pos: Vec2, state: &mut EditorState, _ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        if let Some(stroke_ptr) = scene.sample_pick(mouse_pos, gl) {
            if let Some(stroke) = state.project.strokes.get(stroke_ptr) {
                state.color = stroke.color;
            }
        }
    }

    fn get_icon(&self) -> &str {
        egui_phosphor::regular::EYEDROPPER
    }

    fn name(&self) -> &str {
        "Color Picker"
    }

    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::I)
    }

}
