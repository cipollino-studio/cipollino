
use crate::editor::EditorState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ToolPanel {

}

impl ToolPanel {

    pub fn new() -> Self {
        ToolPanel {  }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        state.curr_tool.clone().borrow_mut().tool_panel(ui, state);
    }

}
