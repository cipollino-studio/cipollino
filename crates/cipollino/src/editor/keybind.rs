
use crate::util::ui::keybind::consume_shortcut;

use super::prefs::{UserPref, UserPrefs};

pub trait Keybind : UserPref<Type = egui::KeyboardShortcut> + Sized {

    fn display_name() -> &'static str;

    fn consume(ui: &mut egui::Ui, user_prefs: &mut UserPrefs) -> bool {
        consume_shortcut(ui, &user_prefs.get::<Self>()) 
    }

}

#[macro_export]
macro_rules! keybind {
    ($name: ident, $display_name: literal, $modifiers: ident, $key: ident) => {
        pub struct $name; 

        impl crate::editor::prefs::UserPref for $name {
            type Type = egui::KeyboardShortcut;

            fn default() -> egui::KeyboardShortcut {
                egui::KeyboardShortcut::new(egui::Modifiers::$modifiers, egui::Key::$key)
            }

            fn name() -> &'static str {
                stringify!($name)
            }
        }

        impl crate::editor::keybind::Keybind for $name {

            fn display_name() -> &'static str {
                $display_name
            }

        }
    };
}

keybind!(UndoKeybind, "Undo", COMMAND, Z);
keybind!(RedoKeybind, "Redo", COMMAND, Y);
keybind!(DeleteKeybind, "Delete", NONE, X);

keybind!(PlayKeybind, "Play", NONE, Space);
keybind!(NewFrameKeybind, "New Frame", NONE, K);
keybind!(StepBackKeybind, "Step Back", NONE, Comma);
keybind!(StepForwardKeybind, "Step Forward", NONE, Period);
keybind!(PrevFrameKeybind, "Previous Frame", COMMAND, Comma);
keybind!(NextFrameKeybind, "Next Frame", COMMAND, Period);
