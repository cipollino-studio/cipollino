
pub fn draggable_label<P>(ui: &mut egui::Ui, text: &str, payload: P) -> egui::Response where P: std::marker::Send + std::marker::Sync + 'static {
    draggable_widget(ui, payload, |ui| {
    let label = egui::Label::new(text).selectable(false).sense(egui::Sense::click());
        let resp = ui.add(label);
        (resp.clone(), resp)
    })
}

pub fn draggable_widget<F, P, R>(ui: &mut egui::Ui, payload: P, mut add_contents: F) -> R
    where F: FnMut(&mut egui::Ui) -> (R, egui::Response),
          P: std::marker::Send + std::marker::Sync + 'static {
    let id = ui.next_auto_id();
    let dragged = ui.memory(|mem| mem.is_being_dragged(id));
    if dragged {
        ui.dnd_drag_source(id, payload, |ui| {
            add_contents(ui)
        }).inner.0
    } else {
        let (val, resp) = add_contents(ui);
        if resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.delta()).length() > 1.0 {
            ui.memory_mut(|mem| mem.set_dragged_id(id));
            egui::DragAndDrop::set_payload(ui.ctx(), payload);
        } else {
            ui.memory_mut(|mem| {
                if mem.dragged_id() == Some(id) {
                    mem.stop_dragging();
                }
            })
        } 
        val
    }
}

pub fn label_size(ui: &mut egui::Ui, label: egui::Label) -> egui::Vec2 {
    let (_, galley, _) = label.layout_in_ui(ui);
    let text_rect = galley.rect;
    text_rect.size()
}

pub fn dnd_drop_zone_setup_colors(ui: &mut egui::Ui) -> (egui::Color32, egui::Color32) {
    let style = ui.style_mut();
    let init_inactive_bg_fill = std::mem::replace(&mut style.visuals.widgets.inactive.bg_fill, style.visuals.window_fill);
    let init_active_bg_fill = std::mem::replace(&mut style.visuals.widgets.active.bg_fill, style.visuals.window_fill);
    (init_inactive_bg_fill, init_active_bg_fill)
}

pub fn dnd_drop_zone_reset_colors(ui: &mut egui::Ui, colors: (egui::Color32, egui::Color32)) {
    let (init_inactive_bg_fill, init_active_bg_fill) = colors; 
    let style = ui.style_mut();
    style.visuals.widgets.inactive.bg_fill = init_inactive_bg_fill;
    style.visuals.widgets.active.bg_fill = init_active_bg_fill;
}