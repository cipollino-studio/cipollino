
use crate::editor::state::EditorState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ToolPanel {

}

impl ToolPanel {

    pub fn new() -> Self {
        ToolPanel {  }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        state.curr_tool.clone().write().unwrap().tool_panel(ui, state);
    }

}
