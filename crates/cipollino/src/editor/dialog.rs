
use egui::Window;
use unique_type_id::{TypeId, UniqueTypeId};
use super::{state::EditorState, EditorSystems};

pub trait Dialog: UniqueTypeId<u64> {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) -> bool;
    fn title(&self, state: &EditorState) -> String;

    // Returns true if only one of this type of dialog can exist at once
    fn unique_dialog() -> bool {
        false
    }

    fn show_title(&self) -> bool {
        true
    }

    fn resizable(&self) -> bool {
        true
    }

    fn anchor(&self) -> Option<egui::Align2> {
        None
    }

    fn margin(&self) -> Option<egui::Margin> {
        None
    }

}

// Shim trait to get around Rust's ban on dyn traits with non-self functions
trait DialogDyn {

    fn render_dyn(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) -> bool;
    fn title_dyn(&self, state: &EditorState) -> String;
    fn show_title_dyn(&self) -> bool;
    fn resizable_dyn(&self) -> bool;
    fn anchor_dyn(&self) -> Option<egui::Align2>;
    fn margin_dyn(&self) -> Option<egui::Margin>;

    fn unique_dialog_dyn(&self) -> bool;
    fn type_id_dyn(&self) -> TypeId<u64>;

}

impl<T: Dialog> DialogDyn for T {

    fn render_dyn(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) -> bool {
        self.render(ui, state, systems)
    }

    fn title_dyn(&self, state: &EditorState) -> String {
        self.title(state)
    }

    fn show_title_dyn(&self) -> bool {
        self.show_title()
    }

    fn resizable_dyn(&self) -> bool {
        self.resizable()
    }

    fn anchor_dyn(&self) -> Option<egui::Align2> {
        self.anchor()
    }

    fn margin_dyn(&self) -> Option<egui::Margin> {
        self.margin()        
    }

    fn unique_dialog_dyn(&self) -> bool {
        Self::unique_dialog()
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

    pub fn open_dialogs(&mut self, dialogs_to_open: DialogsToOpen) {
        for dialog in dialogs_to_open.dialogs {
            self.open_dialog(dialog);
        }
    }

    fn open_dialog(&mut self, dialog: Box<dyn DialogDyn>) {
        if dialog.unique_dialog_dyn() {
            let new_type_id = dialog.type_id_dyn();
            for dialog in &self.dialogs {
                if dialog.dialog.type_id_dyn() == new_type_id {
                    return;
                }
            }
        }
        self.dialogs.push(DialogInstance {
            id: egui::Id::new(self.id_counter),
            dialog: dialog 
        });

        self.id_counter += 1;
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut EditorState, systems: &mut EditorSystems) {
        let mut closed_dialogs = Vec::new();

        for (idx, dialog) in &mut self.dialogs.iter_mut().enumerate() {
            let mut open = true;
            let mut close = false;
            let mut window = Window::new(dialog.dialog.title_dyn(&state))
                .id(dialog.id)
                .open(&mut open)
                .collapsible(false)
                .title_bar(dialog.dialog.show_title_dyn())
                .resizable(dialog.dialog.resizable_dyn());
        
            if let Some(anchor) = dialog.dialog.anchor_dyn() {
                window = window.anchor(anchor, egui::Vec2::ZERO);
            }
            if let Some(margin) = dialog.dialog.margin_dyn() {
                window = window.frame(egui::Frame {
                    inner_margin: margin,
                    ..egui::Frame::window(&*ctx.style())
                });
            }

            window.show(ctx, |ui| {
                if dialog.dialog.render_dyn(ui, state, systems) {
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

pub struct DialogsToOpen { 
    dialogs: Vec<Box<dyn DialogDyn>>
}

impl DialogsToOpen {

    pub fn new() -> Self {
        Self {
            dialogs: Vec::new()
        }
    }

    pub fn open_dialog<T>(&mut self, dialog: T) where T: Dialog + 'static {
        self.dialogs.push(Box::new(dialog));
    }
    
}
