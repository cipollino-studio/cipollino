

use crate::{editor::{selection::Selection, EditorState}, project::{action::Action, frame::Frame, layer::{Layer, LayerKind}, obj::{child_obj::ChildObj, ObjPtr}, sound_instance::SoundInstance}};

pub mod controls;
pub mod header;
pub mod layers;
pub mod frame_area;

const HIGHLIGHT: egui::Color32 = egui::Color32::from_rgba_premultiplied(13, 13, 13, 1);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimelinePanel {
    scroll_x: f32,
    scroll_w: f32,
    scroll_y: f32,
    scroll_h: f32,

    #[serde(skip)]
    set_gfx_len_action: Action,
    #[serde(skip)]
    frame_drag: egui::Vec2,
    #[serde(skip)]
    frame_shift: i32,
    #[serde(skip)]
    frame_drag_action: Option<Action>,
    #[serde(skip)]
    sound_drag: f32,
    #[serde(skip)]
    sound_drag_action: Option<Action>,
    #[serde(skip)]
    prev_mouse_down: bool,
    #[serde(skip)]
    mouse_down_frame: ObjPtr<Frame>,
    #[serde(skip)]
    mouse_down_sound: ObjPtr<SoundInstance>,
    #[serde(skip)]
    layer_editing_name: ObjPtr<Layer>,
    #[serde(skip)]
    layer_edit_curr_name: String
}

pub enum FrameGridRowKind {
    AnimationLayer(ObjPtr<Layer>),
    AudioLayer(ObjPtr<Layer>)
}

pub struct FrameGridRow {
    kind: FrameGridRowKind,
    idx: usize
}

impl TimelinePanel {

    pub fn new() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_w: 0.0,
            scroll_y: 0.0,
            scroll_h: 0.0,
            set_gfx_len_action: Action::new(),
            frame_drag: egui::vec2(0.0, 0.0),
            frame_shift: 0,
            frame_drag_action: None,
            sound_drag: 0.0,
            sound_drag_action: None,
            prev_mouse_down: false,
            mouse_down_frame: ObjPtr::null(),
            mouse_down_sound: ObjPtr::null(),
            layer_editing_name: ObjPtr::null(),
            layer_edit_curr_name: "".to_owned()
        }
    }

    fn calc_grid_rows(&mut self, state: &mut EditorState) -> Vec<FrameGridRow> {
        let gfx = state.project.graphics.get(state.open_graphic).unwrap();
        let mut grid_rows = Vec::new();
        for (idx, layer) in gfx.layers.iter().enumerate() {
            grid_rows.push(FrameGridRow {
                kind: match layer.get(&state.project).kind {
                    LayerKind::Animation => FrameGridRowKind::AnimationLayer(layer.make_ptr()),
                    LayerKind::Audio => FrameGridRowKind::AudioLayer(layer.make_ptr()) 
                }, 
                idx: idx 
            });
        }
        grid_rows
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {

        if let None = state.project.graphics.get(state.open_graphic) {
            ui.centered_and_justified(|ui| {
                ui.label("No Graphic Open");
            });
            return;
        };

        let grid_rows = self.calc_grid_rows(state);

        let gfx = state.project.graphics.get(state.open_graphic).unwrap();

        let frame_w = 10.0;
        let frame_h = 15.0;
        let sidebar_w = 150.0;

        let n_frames = ((ui.available_width() - sidebar_w) / frame_w) as i32 + (gfx.len as i32) - 2;
        let n_frames = 5 * (n_frames / 5) + 4;

        let no_margin = egui::Frame { inner_margin: egui::Margin::same(0.0), ..Default::default()};
        ui.visuals_mut().clip_rect_margin = 0.0;

        egui::TopBottomPanel::top(ui.next_auto_id())
            .resizable(false)
            .exact_height(22.)
            .show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    controls::timeline_controls(self, ui, state);
                });
            }); 

        let response = egui::CentralPanel::default()
            .frame(no_margin)
            .show_inside(ui, |ui| {
            let header_height = 24.0;
            let hovering_layers = egui::SidePanel::left(ui.next_auto_id())
                .resizable(false)
                .exact_width(sidebar_w)
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    egui::TopBottomPanel::top(ui.next_auto_id())
                        .resizable(false)
                        .exact_height(header_height)
                        .show_separator_line(false)
                        .frame(no_margin)
                        .show_inside(ui, |_ui| {

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
                                self.scroll_y -= ui.input(|i| i.smooth_scroll_delta.y);
                                self.scroll_y = self.scroll_y.clamp(0.0, self.scroll_h);
                            }
                            let scroll_area = egui::ScrollArea::vertical()
                                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                                .vertical_scroll_offset(self.scroll_y);
                            scroll_area.show(ui, |ui| {
                                layers::layers(self, ui, frame_h, state, &grid_rows, sidebar_w);
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

            egui::TopBottomPanel::top(ui.next_auto_id())
                .resizable(false)
                .exact_height(header_height)
                .frame(no_margin)
                .show_inside(ui, |ui| {
                    if hovering_frame_num_bar {
                        self.scroll_x -= ui.input(|i| i.smooth_scroll_delta.x);
                        self.scroll_x = self.scroll_x.clamp(0.0, self.scroll_w);
                    }
                    let scroll_area = egui::ScrollArea::horizontal()
                        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                        .horizontal_scroll_offset(self.scroll_x);
                    scroll_area.show(ui, |ui| {
                        header::header(self, ui, frame_w, n_frames, state, header_height);
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
                        frame_area::frames(self, ui, frame_w, frame_h, state, n_frames, &grid_rows);
                        self.scroll_x = ui.clip_rect().left() - ui.min_rect().left(); 
                        self.scroll_x = self.scroll_x.clamp(0.0, self.scroll_w);
                        self.scroll_w = (ui.min_rect().width() - ui.clip_rect().width()).max(0.0);

                        self.scroll_y = ui.clip_rect().top() - ui.min_rect().top(); 
                        self.scroll_y = self.scroll_y.clamp(0.0, self.scroll_h);
                        self.scroll_h = (ui.min_rect().height() - ui.clip_rect().height()).max(0.0);
                    });

                });
        }).response;

        if response.clicked_elsewhere() && state.selection.is_frames() {
            state.selection.clear();
        }

        // Deleting frames
        let delete_shortcut = state.delete_shortcut();
        if let Selection::Timeline(frames, sounds) = &mut state.selection {
            if ui.input_mut(|i| i.consume_shortcut(&delete_shortcut)) {
                let mut action = Action::new();
                for frame_ptr in frames {
                    if let Some(act) = Frame::delete(&mut state.project, *frame_ptr) {
                        action.add(act);
                    }
                }
                for sound_ptr in sounds {
                    if let Some(act) = SoundInstance::delete(&mut state.project, *sound_ptr) {
                        action.add(act);
                    }
                }
                state.selection.clear();
                state.actions.add(action);
                state.reset_tool();
            }
        }

    } 

}

pub fn new_frame(state: &mut EditorState) -> Option<()> {
    let layer = state.project.layers.get(state.active_layer)?;
    if layer.kind != LayerKind::Animation {
        return None;
    }
    let time = state.frame();
    if let None = layer.get_frame_exactly_at(&state.project, time) {
        if let Some((_, act)) = Frame::add(&mut state.project, state.active_layer, Frame {
            layer: state.active_layer,
            time,
            strokes: Vec::new()
        }) {
            state.actions.add(Action::from_single(act));
        }
    }
    None
}

pub fn prev_keyframe(state: &mut EditorState) {
    if let Some(layer) = state.project.layers.get(state.active_layer) {
        if let Some(frame) = layer.get_frame_before(&state.project, state.frame()) {
            state.time = ((frame.get(&state.project).time as f32) * state.frame_len() / state.sample_len()).floor() as i64;
        }
    }
}

pub fn next_keyframe(state: &mut EditorState) {
    if let Some(layer) = state.project.layers.get(state.active_layer) {
        if let Some(frame) = layer.get_frame_after(&state.project, state.frame()) {
            state.time = ((frame.get(&state.project).time as f32) * state.frame_len() / state.sample_len()).floor() as i64;
        }
    }
}