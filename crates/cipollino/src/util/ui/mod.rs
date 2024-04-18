
pub mod color;
pub mod path;
pub mod dnd;
pub mod layout;
pub mod keybind;

use std::ops::RangeInclusive;
pub fn drag_value<N>(ui: &mut egui::Ui, label: &str, val: &mut N, range: RangeInclusive<N>, change: Option<&mut (bool, bool)>) where N: egui::emath::Numeric {
    ui.horizontal(|ui| {
        let initial_val = *val;
        if !label.is_empty() {
            ui.label(format!("{}:", label));
        }
        let val_drag = ui.add(egui::DragValue::new(val).clamp_range(range).update_while_editing(false));
        if val_drag.dragged() {
            if let Some((edit, _set)) = change {
                *edit = true;
            }
        }
        if val_drag.drag_released() || (!val_drag.dragged() && *val != initial_val) {
            if let Some((edit, set)) = change {
                *edit = true;
                *set = true;
            }
        }
    });
}
