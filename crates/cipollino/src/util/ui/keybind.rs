
pub fn consume_shortcut(ui: &mut egui::Ui, shortcut: &egui::KeyboardShortcut) -> bool {
    let res = ui.input_mut(|i| i.consume_shortcut(shortcut)) && !ui.memory(|mem| mem.focus().is_some());
    if res {
        ui.ctx().request_repaint();
    }
    res
}
