
use unique_type_id::{TypeId, UniqueTypeId};
use super::state::EditorState;

pub trait Dialog: UniqueTypeId<u64> {

    // Returns true if only one of this type of dialog can exist at once
    fn unique_dialog() -> bool {
        false
    }

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool;
    fn title(&self, state: &EditorState) -> String;

}

// Shim trait to get around Rust's ban on dyn traits with non-self functions
trait DialogDyn {

    fn render_dyn(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool;
    fn title_dyn(&self, state: &EditorState) -> String;
    fn type_id_dyn(&self) -> TypeId<u64>;

}

impl<T: Dialog> DialogDyn for T {

    fn render_dyn(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool {
        self.render(ui, state)
    }

    fn title_dyn(&self, state: &EditorState) -> String {
        self.title(state)
    }

    fn type_id_dyn(&self) -> TypeId<u64> {
        Self::id()
    }

}

struct DialogInstance {
    pub dialog: Box<dyn DialogDyn>,
    pub id: egui::Id
}

pub struct DialogManager {
    dialogs: Vec<DialogInstance>,
    id_counter: u64
}

impl DialogManager {

    pub fn new() -> Self {
        Self {
            dialogs: Vec::new(),
            id_counter: 0
        }
    }

    pub fn open_dialog<T>(&mut self, dialog: T) where T: Dialog + 'static {
        if T::unique_dialog() {
            let new_type_id = T::id();
            for dialog in &self.dialogs {
                if dialog.dialog.type_id_dyn() == new_type_id {
                    return;
                }
            }
        }
        self.dialogs.push(DialogInstance {
            id: egui::Id::new(self.id_counter),
            dialog: Box::new(dialog)
        });

        self.id_counter += 1;
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let mut closed_dialogs = Vec::new();

        for (idx, dialog) in &mut self.dialogs.iter_mut().enumerate() {
            let mut open = true;
            let mut close = false;
            egui::Window::new(dialog.dialog.title_dyn(&state))
                .id(dialog.id)
                .open(&mut open)
                .collapsible(false)
                .show(ctx, |ui| {
                    if dialog.dialog.render_dyn(ui, state) {
                        close = true; 
                    }
            });
            if close {
                open = false;
            }

            if !open {
                closed_dialogs.push(idx);
            }
        }

        for close_idx in closed_dialogs.iter().rev() {
            self.dialogs.remove(*close_idx);
        }
    }

}
