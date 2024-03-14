
use egui::Vec2;

use crate::editor::state::EditorState;

use super::TimelinePanel;

pub fn header(_timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_w: f32, n_frames: i32, state: &mut EditorState, header_height: f32) {
    let (rect, response) = ui.allocate_exact_size(Vec2::new((n_frames as f32) * frame_w, header_height), egui::Sense::click_and_drag());
    let tl = rect.left_top(); 
    if response.dragged() || response.drag_started() || response.clicked() {
        if let Some(mouse_pos) = response.hover_pos() {
            let mx = mouse_pos.x - rect.left();
            let frame = (mx / frame_w).floor();
            state.time = (frame * state.frame_len() / state.sample_len() + 5.0).floor() as i64;
            state.pause();
        }
    }
    for i in (4..n_frames).step_by(5) {
        let pos = tl + Vec2::new((i as f32 + 0.5) * frame_w, 4.0);
        let rect = egui::Rect::from_min_max(pos, pos);
        ui.put(rect, egui::Label::new(format!("{}", i + 1)).wrap(false).selectable(false));
    }
    ui.painter().rect(
        egui::Rect::from_min_max(tl + Vec2::new((state.frame() as f32) * frame_w, 0.0), tl + Vec2::new((state.frame() as f32 + 1.0) * frame_w, header_height - 2.0)),
        0.0,
        egui::Color32::from_rgba_unmultiplied(125, 125, 255, 10),
        egui::Stroke::new(1.0, egui::Color32::from_rgb(125, 125, 255)));
}
