
use crate::{editor::EditorSystems, keybind, use_tool_state_machine};

use self::neutral::Neutral;

use super::{state_machine::ToolStateMachine, Tool};

mod neutral;
mod cut;

pub struct Scissors {
    state: ToolStateMachine
}

impl Scissors {

    pub fn new() -> Self {
        Self {
            state: ToolStateMachine::new(Neutral {})
        } 
    }

}

impl Tool for Scissors {

    use_tool_state_machine!(state);

    fn get_icon(&self) -> &str {
        egui_phosphor::regular::SCISSORS
    }

    fn name(&self) -> &str {
        "Scissors"
    }

    fn shortcut(&self, systems: &mut EditorSystems) -> egui::KeyboardShortcut {
        systems.prefs.get::<ScissorsToolKeybind>()
    }

}

keybind!(ScissorsToolKeybind, "Scissors", COMMAND, T);
