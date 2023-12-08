
use egui::Vec2;

use crate::{editor::EditorState, project::{action::Action, graphic::GraphicData}};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimelinePanel {
    scroll_x: f32,
    scroll_w: f32,
    scroll_y: f32,
    scroll_h: f32,

    #[serde(skip)]
    set_gfx_len_action: Action
}

impl TimelinePanel {

    pub fn new() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_w: 0.0,
            scroll_y: 0.0,
            scroll_h: 0.0,
            set_gfx_len_action: Action::new()
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if let None = state.project.graphics.get(&state.open_graphic.unwrap_or(0)) {
            ui.centered_and_justified(|ui| {
                ui.label("No Graphic Open");
            });
            return;
        };
        let gfx_key = state.open_graphic.unwrap();

        let frame_w = 10.0;
        let frame_h = 15.0;
        let sidebar_w = 100.0;

        let n_frames = ((ui.available_width() - sidebar_w) / frame_w) as i32 + (state.project.graphics.get(&gfx_key).unwrap().data.len as i32) - 2;
        let n_frames = 5 * (n_frames / 5) + 4;

        let no_margin = egui::Frame { inner_margin: egui::Margin::same(0.0), ..Default::default()};
        ui.visuals_mut().clip_rect_margin = 0.0;
        let highlight = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 1);

        egui::TopBottomPanel::top("timeline_controls")
            .resizable(false)
            .exact_height(22.)
            .show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    self.timeline_controls(ui, state, gfx_key);
                });
            }); 

        egui::CentralPanel::default()
            .frame(no_margin)
            .show_inside(ui, |ui| {
            let header_height = 24.0;
            let hovering_layers = egui::SidePanel::left("timeline_sidebar")
                .resizable(false)
                .exact_width(sidebar_w)
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    egui::TopBottomPanel::top("timeline_layer_header")
                        .resizable(false)
                        .exact_height(header_height)
                        .show_separator_line(false)
                        .frame(no_margin)
                        .show_inside(ui, |_ui| {
                            // ui.label("EYE/LOCK ICONS?");
                        });
                    let hovering_layers = if let Some(pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                        ui.available_rect_before_wrap().contains(pos)
                    } else {
                        false
                    };
                    egui::CentralPanel::default()
                        .frame(no_margin)
                        .show_inside(ui, |ui| {
                            if hovering_layers {
                                self.scroll_y -= ui.input(|i| i.scroll_delta.y);
                                self.scroll_y = self.scroll_y.clamp(0.0, self.scroll_h);
                            }
                            let scroll_area = egui::ScrollArea::vertical()
                                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                .vertical_scroll_offset(self.scroll_y);
                            scroll_area.show(ui, |ui| {
                                self.layers(ui, frame_h, state, highlight);
                            });
                        });
                    hovering_layers
                }).inner; 

            let hovering_frame_num_bar = if let Some(pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                let mut frame_num_bar_rect = ui.available_rect_before_wrap();
                frame_num_bar_rect.set_height(header_height);
                frame_num_bar_rect.contains(pos)
            } else {
                false
            };

            egui::TopBottomPanel::top("timeline_frame_numbers")
                .resizable(false)
                .exact_height(header_height)
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    if hovering_frame_num_bar {
                        self.scroll_x -= ui.input(|i| i.scroll_delta.x);
                        self.scroll_x = self.scroll_x.clamp(0.0, self.scroll_w);
                    }
                    let scroll_area = egui::ScrollArea::horizontal()
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .horizontal_scroll_offset(self.scroll_x);
                    scroll_area.show(ui, |ui| {
                        self.frame_numbers(ui, frame_w, n_frames, state, header_height);
                    });
                });
            egui::CentralPanel::default()
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    let mut scroll_area = egui::ScrollArea::both()
                        .auto_shrink([false, false]);
                    if hovering_frame_num_bar {
                        scroll_area = scroll_area.horizontal_scroll_offset(self.scroll_x);
                    }
                    if hovering_layers {
                        scroll_area = scroll_area.vertical_scroll_offset(self.scroll_y);
                    }
                    scroll_area.show(ui, |ui| {
                        self.frames(ui, frame_w, frame_h, state, highlight, n_frames);
                        self.scroll_x = ui.clip_rect().left() - ui.min_rect().left(); 
                        self.scroll_x = self.scroll_x.clamp(0.0, self.scroll_w);
                        self.scroll_w = (ui.min_rect().width() - ui.clip_rect().width()).max(0.0);

                        self.scroll_y = ui.clip_rect().top() - ui.min_rect().top(); 
                        self.scroll_y = self.scroll_y.clamp(0.0, self.scroll_h);
                        self.scroll_h = (ui.min_rect().height() - ui.clip_rect().height()).max(0.0);
                    });

                });
        });

    } 

    pub fn timeline_controls(&mut self, ui: &mut egui::Ui, state: &mut EditorState, gfx: u64) {
        
        if ui.button("+").clicked() {
            if let Some((key, act)) = state.project.add_layer(gfx, "Layer".to_owned()) {
                state.actions.add(Action::from_single(act));
                state.active_layer = key;
            }
        }

        if ui.button("<<").clicked() {
            state.time = 0.0;
            state.playing = false;
        }
        if ui.button("*<").clicked() {
            prev_keyframe(state);
            state.playing = false;
        }
        if ui.button(if state.playing { "||" } else { ">" }).clicked() {
            state.playing = !state.playing; 
        }
        if ui.button(">*").clicked() {
            next_keyframe(state);
            state.playing = false;
        }
        if ui.button(">>").clicked() {
            let gfx = state.project.graphics.get(&gfx).unwrap();
            state.time = (gfx.data.len as f32 - 1.0) * state.frame_len();
            state.playing = false;
        }

        let gfx_key = state.open_graphic.unwrap();
        let gfx = state.project.graphics.get(&gfx_key).unwrap();
        let mut len = gfx.data.len; 
        ui.label("Graphic length: ");
        let gfx_len_drag = ui.add(egui::DragValue::new(&mut len).clamp_range(1..=1000000).update_while_editing(false));
        let len_changed = len != gfx.data.len;
        if len_changed {
            if let Some(act) = state.project.set_graphic_data(gfx_key, GraphicData {
                len,
                ..gfx.data.clone()
            }) {
                self.set_gfx_len_action.add(act);
            }
        }
        if gfx_len_drag.drag_released() || (!gfx_len_drag.dragged() && len_changed) {
            state.actions.add(std::mem::replace(&mut self.set_gfx_len_action, Action::new()));
        }

        ui.label("Onion skin:");
        ui.add(egui::DragValue::new(&mut state.onion_before).clamp_range(0..=10));
        ui.add(egui::DragValue::new(&mut state.onion_after).clamp_range(0..=10));

    }

    pub fn frame_numbers(&mut self, ui: &mut egui::Ui, frame_w: f32, n_frames: i32, state: &mut EditorState, header_height: f32) {
        let (rect, response) = ui.allocate_exact_size(Vec2::new((n_frames as f32) * frame_w, header_height), egui::Sense::click_and_drag());
        let tl = rect.left_top(); 
        if response.dragged() || response.clicked() {
            if let Some(mouse_pos) = response.hover_pos() {
                let mx = mouse_pos.x - rect.left();
                let frame = (mx / frame_w).floor();
                state.time = frame * (1.0 / 24.0);
                state.playing = false;
            }
        }
        for i in (4..n_frames).step_by(5) {
            let pos = tl + Vec2::new((i as f32 + 0.5) * frame_w, 4.0);
            let rect = egui::Rect::from_min_max(pos, pos);
            ui.put(rect, egui::Label::new(format!("{}", i + 1)).wrap(false));
        }
        ui.painter().rect(
            egui::Rect::from_min_max(tl + Vec2::new((state.frame() as f32) * frame_w, 0.0), tl + Vec2::new((state.frame() as f32 + 1.0) * frame_w, header_height - 2.0)),
            0.0,
            egui::Color32::from_rgba_unmultiplied(125, 125, 255, 10),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(125, 125, 255)));
    }

    pub fn layers(&mut self, ui: &mut egui::Ui, frame_h: f32, state: &mut EditorState, highlight: egui::Color32) {
        let gfx = state.project.graphics.get(&state.open_graphic.unwrap()).unwrap();
        let mut i = 0;

        let mut delete_layer = None;

        let (rect, _response) = ui.allocate_exact_size(Vec2::new(100.0, (gfx.layers.len() as f32) * frame_h), egui::Sense::click());
        let tl = rect.left_top(); 
        for layer_key in gfx.layers.iter() {
            if let Some(layer) = state.project.layers.get(&layer_key) {
                let layer_name_tl = tl + Vec2::new(0.0, frame_h * (i as f32)); 
                let layer_name_br = layer_name_tl + Vec2::new(100.0, frame_h); 
                let rect = egui::Rect::from_min_max(layer_name_tl, layer_name_br);
                if *layer_key == state.active_layer {
                    ui.painter().rect(rect, 0.0, highlight, egui::Stroke::NONE);
                }
                let layer_name_response = ui.put(rect, egui::Label::new(layer.data.name.clone()).sense(egui::Sense::click()))
                    .context_menu(|ui| {
                        if ui.button("Delete").clicked() {
                            delete_layer = Some(*layer_key); 
                        }
                    });
                if layer_name_response.clicked() {
                    state.active_layer = *layer_key;
                }
            }
            i += 1;
        }
        
        if let Some(key) = delete_layer {
            if let Some(acts) = state.project.delete_layer(key) {
                state.actions.add(Action::from_list(acts));
            }
        }
    }

    pub fn frames(&mut self, ui: &mut egui::Ui, frame_w: f32, frame_h: f32, state: &mut EditorState, highlight: egui::Color32, n_frames: i32) {

        let gfx = state.project.graphics.get(&state.open_graphic.unwrap()).unwrap();

        let total_height = ui.available_height().max(frame_h * (gfx.layers.len() as f32));

        let (rect, _response) = ui.allocate_exact_size(Vec2::new((n_frames as f32) * frame_w, ((gfx.layers.len() as f32) * frame_h).max(ui.available_height())), egui::Sense::click_and_drag());
        let win_tl = rect.left_top(); 
        ui.painter().rect(
            egui::Rect::from_min_max(rect.left_top(), rect.right_bottom() + Vec2::new(0.0, ui.available_height())),
            0.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 40),
            egui::Stroke::NONE);

        // Active layer highlight
        let mut y = 0.0;
        for layer_key in gfx.layers.iter() {
            if *layer_key == state.active_layer {
                ui.painter().rect(
                    egui::Rect::from_min_max(win_tl + Vec2::new(0.0, y * frame_h), win_tl + Vec2::new((n_frames as f32) * frame_w, (y + 1.0) * frame_h)),
                    0.0,
                    highlight,
                    egui::Stroke::NONE);
            }
            y += 1.0;
        }

        // Frame interval highlight
        for x in (4..n_frames).step_by(5) {
            ui.painter().rect(
                egui::Rect::from_min_max(win_tl + Vec2::new((x as f32) * frame_w, 0.0), win_tl + Vec2::new((x as f32 + 1.0) * frame_w, rect.height())),
                0.0,
                highlight,
                egui::Stroke::NONE);
        }

        // Frame dots
        let mut y = 0.0;
        for layer_key in gfx.layers.iter() {
            let layer = state.project.layers.get(layer_key).unwrap();
            for frame_key in &layer.frames {
                let frame = state.project.frames.get(frame_key).unwrap();
                ui.painter().circle(
                    win_tl + Vec2::new((frame.data.time as f32 + 0.5) * frame_w, (y + 0.5) * frame_h),
                    frame_w * 0.3,
                    ui.visuals().text_color(),
                    egui::Stroke::NONE);
            }
            y += 1.0;
        }

        // Playhead
        ui.painter().vline(rect.left() + (state.frame() as f32 + 0.5) * frame_w, egui::Rangef::new(rect.top(), rect.top() + total_height), egui::Stroke::new(1.0, egui::Color32::from_rgb(125, 125, 255)));

        // After graphic end shadow realm
        let darken = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100);
        ui.painter().rect(
            egui::Rect::from_min_max(win_tl + Vec2::new((gfx.data.len as f32) * frame_w, 0.0), rect.max),
            0.0,
            darken,
            egui::Stroke::NONE);

    }

}

pub fn new_frame(state: &mut EditorState) -> Option<()> {
    if let Some(_layer) = state.project.layers.get(&state.active_layer) { 
        if let None = state.project.get_frame_exactly_at(state.active_layer, state.frame()) {
            if let Some((_key, act)) = state.project.add_frame(state.active_layer, state.frame()) {
                state.actions.add(Action::from_single(act));
            }
        }
    }
    None
}

pub fn prev_keyframe(state: &mut EditorState) -> Option<()> {
    if let Some(_layer) = state.project.layers.get(&state.active_layer) {
        if let Some(frame) = state.project.get_frame_before(state.active_layer, state.frame()) {
            state.time = (state.project.frames.get(&frame).unwrap().data.time as f32) * state.frame_len();
        }
    }
    None
}

pub fn next_keyframe(state: &mut EditorState) -> Option<()> {
    if let Some(_layer) = state.project.layers.get(&state.active_layer) {
        if let Some(frame) = state.project.get_frame_after(state.active_layer, state.frame()) {
            state.time = (state.project.frames.get(&frame).unwrap().data.time as f32) * state.frame_len();
        }
    }
    None
}