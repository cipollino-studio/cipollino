
use egui::{vec2, Vec2};

use crate::{editor::{selection::Selection, state::EditorState}, project::{action::Action, folder::Folder, frame::Frame, layer::Layer, obj::{child_obj::ChildObj, ObjPtr}, sound_instance::SoundInstance, AssetPtr}};

use super::{FrameGridRow, FrameGridRowKind, TimelinePanel};

impl FrameGridRow {

    fn active_frame_highlight(layer_ptr: ObjPtr<Layer>, state: &mut EditorState, ui: &mut egui::Ui, rect: egui::Rect) {
        // Active layer highlight
        if layer_ptr == state.active_layer {
            ui.painter().rect(
                rect,
                0.0,
                super::HIGHLIGHT,
                egui::Stroke::NONE);
        }
    }

    fn frame_area_layer_animation(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_w: f32, frame_h: f32, state: &mut EditorState, rect: egui::Rect, response: &egui::Response, mouse_went_down: bool, layer_ptr: ObjPtr<Layer>) -> Option<()> {
        Self::active_frame_highlight(layer_ptr, state, ui, rect);

        let layer = state.project.layers.get(layer_ptr)?;

        // If the user clicks anywhere in the layer, select the layer
        if let Some(hover_pos) = response.hover_pos() {
            if rect.contains(hover_pos) && response.clicked() {
                state.active_layer = layer_ptr;
            } 
        }

        // Frame dots
        for frame in &layer.frames {
            let dot_pos = rect.left_top() + Vec2::new((frame.get(&state.project).time as f32 + 0.5) * frame_w, 0.5 * frame_h);
            let frame_rect = egui::Rect::from_center_size(dot_pos, egui::Vec2::new(frame_w, frame_h));
            ui.painter().circle(
                dot_pos, 
                frame_w * 0.3,
                ui.visuals().text_color(),
                egui::Stroke::NONE);
            if state.selection.frame_selected(frame.make_ptr()) {
                ui.painter().rect_stroke(frame_rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(125, 125, 255))); 
            }
            if let Some(hover_pos) = response.hover_pos() {
                if frame_rect.contains(hover_pos) {
                    if response.clicked() {
                        state.selection.select_frame_inverting(frame.make_ptr());
                    } 
                    if mouse_went_down {
                        timeline.mouse_down_frame = frame.make_ptr();
                    }
                }
            }
        }

        None
    }

    fn frame_area_layer_audio(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_w: f32, frame_h: f32, state: &mut EditorState, rect: egui::Rect, response: &egui::Response, mouse_went_down: bool, layer_ptr: ObjPtr<Layer>) -> Option<()> {
        Self::active_frame_highlight(layer_ptr, state, ui, rect);

        let layer = state.project.layers.get(layer_ptr)?;

        // Sound instances
        for sound_instance_box in &layer.sound_instances {
            let sound_instance = sound_instance_box.get(&state.project);
            let left = rect.left() + (state.frame_rate() * frame_w * state.sample_len()) * (sound_instance.begin as f32);
            let right = rect.left() + (state.frame_rate() * frame_w * state.sample_len()) * (sound_instance.end as f32);
            let sound_instance_rect = egui::Rect::from_min_max(egui::pos2(left, rect.top()), egui::pos2(right, rect.bottom()));
            let stroke = if state.selection.sound_selected(sound_instance_box.make_ptr()) {
                ui.visuals().widgets.active.bg_stroke
            } else {
                egui::Stroke::NONE
            };
            ui.painter().rect(
                sound_instance_rect, 
                0.0,
                egui::Color32::from_rgb(90, 138, 153),
                stroke);

            if let Some(hover_pos) = response.hover_pos() {
                if sound_instance_rect.contains(hover_pos) {
                    if response.clicked() {
                        state.selection.select_sound_inverting(sound_instance_box.make_ptr());
                    }
                    if mouse_went_down {
                        timeline.mouse_down_sound = sound_instance_box.make_ptr();
                    }
                }
            }

            if let Some(audio) = state.project.audio_files.get(&sound_instance.audio.lookup(&state.project)) {
                for x in (left as i32)..(right as i32) {
                    let x = x as f32;
                    let t = (x - left) / (right - left);
                    let volume_sum_idx = (t * (audio.volumes.len() as f32)) as usize;
                    let sum = audio.volumes[volume_sum_idx].powf(0.4);
                    ui.painter().rect_filled(
                        egui::Rect::from_center_size(egui::pos2(x, rect.center().y), egui::vec2(1.0, sum * frame_h)),
                        0.0,
                        egui::Color32::from_rgba_premultiplied(0, 0, 0, 60));
                }
            }
        }

        if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if rect.contains(hover_pos) {
                let begin = (44100.0 * (hover_pos.x - rect.left()) / frame_w / 24.0).floor() as i64;
                if let Some(payload) = response.dnd_hover_payload::<(AssetPtr, ObjPtr<Folder>)>() {
                    if let AssetPtr::Audio(audio) = &(*payload).0 {
                        if let Some(audio) = state.project.audio_files.get(audio) {
                            ui.painter().rect_stroke(
                                egui::Rect::from_min_size(
                                    egui::pos2(hover_pos.x, rect.top()), egui::vec2(frame_w * (audio.samples.len() as f32) * state.sample_len() / state.frame_len(), frame_h)),
                                0.0,
                                ui.visuals().widgets.active.bg_stroke);
                        }
                    }
                }
                if let Some(payload) = response.dnd_release_payload::<(AssetPtr, ObjPtr<Folder>)>() {
                    if let AssetPtr::Audio(audio_file_ptr) = &(*payload).0 {
                        if let Some(audio) = state.project.audio_files.get(audio_file_ptr) {
                            let length = audio.samples.len() as i64;
                            SoundInstance::add(&mut state.project, layer_ptr, SoundInstance {
                                layer: layer_ptr,
                                begin,
                                end: begin + length,
                                audio: audio_file_ptr.clone(),
                            });
                        }
                    }
                }
            }
        }
        None
    }

    fn frame_area(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_w: f32, frame_h: f32, state: &mut EditorState, response: &egui::Response, mouse_went_down: bool, rect: egui::Rect) {
        
        match &self.kind {
            FrameGridRowKind::AnimationLayer(layer) => self.frame_area_layer_animation(timeline, ui, frame_w, frame_h, state, rect, response, mouse_went_down, *layer),
            FrameGridRowKind::AudioLayer(layer) => self.frame_area_layer_audio(timeline, ui, frame_w, frame_h, state, rect, response, mouse_went_down, *layer)
        };
    }

}

pub fn frames(timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_w: f32, frame_h: f32, state: &mut EditorState, n_frames: i32, grid_rows: &Vec<FrameGridRow>) {

    let gfx = state.project.graphics.get(state.open_graphic).unwrap();

    let gfx_len = gfx.len;
    let total_height = ui.available_height().max(frame_h * (gfx.layers.len() as f32));

    let (rect, response) = ui.allocate_exact_size(Vec2::new((n_frames as f32) * frame_w, ((gfx.layers.len() as f32) * frame_h).max(ui.available_height())), egui::Sense::click_and_drag());
    let win_tl = rect.left_top(); 
    let mouse_down = response.is_pointer_button_down_on();
    let mouse_went_down = mouse_down && !timeline.prev_mouse_down;

    ui.painter().rect(
        egui::Rect::from_min_max(rect.left_top(), rect.right_bottom() + Vec2::new(0.0, ui.available_height())),
        0.0,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 40),
        egui::Stroke::NONE);

    if response.clicked() || response.drag_started() {
        if !ui.input(|i| i.modifiers.shift) && state.selection.is_frames() {
            state.selection.clear();
        }
    }

    if mouse_went_down {
        timeline.mouse_down_frame = ObjPtr::null();
    }
    // Frame interval highlight
    for x in (4..n_frames).step_by(5) {
        ui.painter().rect(
            egui::Rect::from_min_max(win_tl + Vec2::new((x as f32) * frame_w, 0.0), win_tl + Vec2::new((x as f32 + 1.0) * frame_w, rect.height())),
            0.0,
            super::HIGHLIGHT,
            egui::Stroke::NONE);
    }

    // Frame area 
    let mut y = 0.0;
    for row in grid_rows {
        let rect = egui::Rect::from_min_size(win_tl + Vec2::new(0.0, y * frame_h), Vec2::new(rect.width(), frame_h));
        row.frame_area(timeline, ui, frame_w, frame_h, state, &response, mouse_went_down, rect);
        y += 1.0;
    }

    if response.drag_started() {
        if state.project.frames.get(timeline.mouse_down_frame).is_some() {
            state.selection.select_frame(timeline.mouse_down_frame)
        }
        if state.project.sound_instances.get(timeline.mouse_down_sound).is_some() {
            state.selection.select_sound(timeline.mouse_down_sound);
        }
        timeline.mouse_down_frame = ObjPtr::null();
        timeline.mouse_down_sound = ObjPtr::null();
    }

    let darken = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 100);

    // Sublayer shadow realm
    ui.painter().rect(
        egui::Rect::from_min_max(win_tl + vec2(0.0, (grid_rows.len() as f32) * frame_h), win_tl + vec2((gfx_len as f32) * frame_w, rect.height())),
        0.0,
        darken,
        egui::Stroke::NONE);

    // After graphic end shadow realm
    ui.painter().rect(
        egui::Rect::from_min_max(win_tl + Vec2::new((gfx_len as f32) * frame_w, 0.0), rect.max),
        0.0,
        darken,
        egui::Stroke::NONE);

    // Playhead
    ui.painter().vline(rect.left() + (state.frame() as f32 + 0.5) * frame_w, egui::Rangef::new(rect.top(), rect.top() + total_height), egui::Stroke::new(1.0, egui::Color32::from_rgb(125, 125, 255)));

    // Frame dragging
    timeline.frame_drag += response.drag_delta();
    if let Selection::Timeline(frames, _) = &state.selection {
        if timeline.frame_drag.x.abs() > frame_w {
            let mut frame_shift_inc = (timeline.frame_drag.x.signum() * (timeline.frame_drag.x.abs() / frame_w).floor()) as i32; 
            for frame in frames {
                state.project.frames.get_then(*frame, |frame| {
                    frame_shift_inc = frame_shift_inc.max(-frame.time);
                });
            }
            timeline.frame_shift += frame_shift_inc;
            if let Some(action) = &timeline.frame_drag_action {
                action.undo(&mut state.project);
            }
            let mut new_action = Action::new();
            let mut frames = frames.clone();
            frames.sort_by(|a_ptr, b_ptr| {
                if let Some(a) = state.project.frames.get(*a_ptr) {
                    if let Some(b) = state.project.frames.get(*b_ptr) {
                        if timeline.frame_shift > 0 {
                            return b.time.cmp(&a.time);
                        } else {
                            return a.time.cmp(&b.time);
                        } 
                    }
                } 
                std::cmp::Ordering::Equal
            });
            for frame_ptr in &frames {
                if let Some(frame) = state.project.frames.get(*frame_ptr) {
                    let time = frame.time;
                    if let Some(acts) = Frame::frame_set_time(&mut state.project, *frame_ptr, time + timeline.frame_shift) {
                        new_action.add_list(acts);
                    }
                }
            }
            timeline.frame_drag_action = Some(new_action);
            timeline.frame_drag.x -= (frame_shift_inc as f32) * frame_w;
        }
    }

    // Sound dragging
    if let Selection::Timeline(_, sounds) = &state.selection {
        if response.drag_delta().x.abs() > 0.0 {
            if let Some(action) = &timeline.sound_drag_action {
                action.undo(&mut state.project);
            }

            timeline.sound_drag += response.drag_delta().x;
            let mut sound_shift = ((timeline.sound_drag / frame_w) * state.frame_len() / state.sample_len()) as i64;
            for sound in sounds {
                if let Some(sound) = state.project.sound_instances.get(*sound) {
                    sound_shift = sound_shift.max(-sound.begin);
                }
            }
            
            let mut new_action = Action::new();
            for sound_ptr in sounds {
                if let Some(sound) = state.project.sound_instances.get(*sound_ptr) {
                    let begin = sound.begin;
                    let end = sound.end;
                    if let Some(act) = SoundInstance::set_begin(&mut state.project, *sound_ptr, begin + sound_shift) {
                        new_action.add(act);
                    }
                    if let Some(act) = SoundInstance::set_end(&mut state.project, *sound_ptr, end + sound_shift) {
                        new_action.add(act);
                    }
                }
            }
            timeline.sound_drag_action = Some(new_action);
        }
    }

    if response.drag_released() {
        let mut total_action = Action::new();
        if let Some(action) = std::mem::replace(&mut timeline.frame_drag_action, None) {
            total_action.add_list(action.actions); 
        }
        if let Some(action) = std::mem::replace(&mut timeline.sound_drag_action, None) {
            total_action.add_list(action.actions); 
        }
        state.actions.add(total_action);
        timeline.frame_shift = 0;
        timeline.frame_drag = egui::Vec2::ZERO;
        timeline.sound_drag = 0.0;
    }

    timeline.prev_mouse_down = mouse_down;

}
