
use std::sync::Arc;

use glam::Vec2;

use crate::{editor::{state::EditorState, EditorSystems}, keybind, panels::scene::ScenePanel, project::obj::obj_list::ObjListTrait};

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

    fn shortcut(&self, systems: &mut EditorSystems) -> egui::KeyboardShortcut {
        systems.prefs.get::<ColorPickerToolKeybind>()
    }

}

keybind!(ColorPickerToolKeybind, "Color Picker", NONE, I);