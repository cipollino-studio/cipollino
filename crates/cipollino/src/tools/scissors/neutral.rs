
use crate::tools::state_machine::ToolState;

use super::cut::Cut;

pub struct Neutral {

} 

impl ToolState for Neutral {

    fn mouse_click(&mut self, mouse_pos: glam::Vec2, _state: &mut crate::editor::state::EditorState, _ui: &mut egui::Ui, _scene: &mut crate::panels::scene::ScenePanel, _gl: &std::sync::Arc<glow::Context>) -> Option<Box<dyn ToolState>> {
        Some(Box::new(Cut::new(mouse_pos)))
    }

}
