
use egui::{vec2, Vec2};

use crate::{editor::{state::EditorState, EditorSystems}, project::{action::Action, layer::{Layer, LayerKind, LayerParent}, obj::{child_obj::ChildObj, ObjPtr}}, util::ui::dnd::{dnd_drop_zone_reset_colors, dnd_drop_zone_setup_colors, draggable_widget}};

use self::layer_property_dialog::LayerPropertyDialog;

use super::{FrameGridRow, TimelinePanel};

mod layer_property_dialog;

impl FrameGridRow {

    fn render_layer_name(&self, layer: &Layer, ui: &mut egui::Ui, rect: &egui::Rect, indent_size: f32, layer_group_triangle_width: f32, open_close_layer: &mut bool) {
        let mut text_rect = rect.clone();
        let horiz_offset = (self.indent as f32) * indent_size + layer_group_triangle_width;
        text_rect.set_left(text_rect.left() + horiz_offset);
        text_rect.set_right(text_rect.right() + horiz_offset);
        let mut child_ui = ui.child_ui(text_rect, egui::Layout::top_down(egui::Align::LEFT));
        child_ui.add(egui::Label::new(layer.name.clone()).selectable(false));

        if layer.kind == LayerKind::Group {
            let mut triangle_rect = rect.clone();
            let horiz_offset = horiz_offset - layer_group_triangle_width;
            triangle_rect.set_width(layer_group_triangle_width + 2.0);
            triangle_rect.set_left(triangle_rect.left() + horiz_offset); 
            triangle_rect.set_right(triangle_rect.right() + horiz_offset); 
            let layer_group_triangle = egui::Label::new(if layer.open { egui_phosphor::regular::CARET_DOWN } else { egui_phosphor::regular::CARET_RIGHT }).selectable(false).sense(egui::Sense::click()); 
            if ui.put(triangle_rect, layer_group_triangle).clicked() {
                *open_close_layer = true;
            }
        }
    }

    fn layer_context_menu(&self, timeline: &mut TimelinePanel, layer: &Layer, response: &egui::Response, set_layer_kind: &mut Option<LayerKind>, delete_layer: &mut bool, systems: &mut EditorSystems) {
        response.context_menu(|ui| {
            if ui.button("Rename").clicked() {
                timeline.layer_editing_name = self.layer;
                timeline.layer_edit_curr_name = layer.name.clone();
                ui.close_menu();
            }
            if layer.kind != LayerKind::Group {
                ui.menu_button("Layer Type", |ui| {
                    if ui.button("Animation").clicked() {
                        *set_layer_kind = Some(LayerKind::Animation);
                        ui.close_menu();
                    }
                    if ui.button("Audio").clicked() {
                        *set_layer_kind = Some(LayerKind::Audio);
                        ui.close_menu(); 
                    }
                });
            }
            if ui.button("Properties").clicked() {
                systems.dialog.open_dialog(LayerPropertyDialog::new(self.layer));
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                *delete_layer = true;
                ui.close_menu();
            }
        });
    } 

    // The show/hide icon on animation layers, mute/unmute on audio layers
    fn render_layer_icons(&self, layer: &Layer, ui: &mut egui::Ui, rect: &egui::Rect, show_hide_layer: &mut bool) {
        let eye_text = if layer.show {
            match layer.kind {
                LayerKind::Animation => egui_phosphor::regular::EYE,
                LayerKind::Audio => egui_phosphor::regular::SPEAKER_HIGH,
                LayerKind::Group => egui_phosphor::regular::EYE
            }
        } else {
            match layer.kind {
                LayerKind::Animation => egui_phosphor::regular::EYE_CLOSED,
                LayerKind::Audio => egui_phosphor::regular::SPEAKER_SLASH,
                LayerKind::Group => egui_phosphor::regular::EYE_CLOSED
            }
        };
        let eye_width = 15.0; 
        let eye_rect = egui::Rect::from_min_size(rect.max - vec2(eye_width + ui.spacing().icon_spacing, rect.height()), vec2(eye_width, rect.height())); 
        let eye_resp = ui.put(eye_rect, egui::Label::new(eye_text).selectable(false).sense(egui::Sense::click()));
        if eye_resp.clicked() {
            *show_hide_layer = true;
        }
    }

    fn render_layer(&self, timeline: &mut TimelinePanel, ui: &mut egui::Ui, rect: egui::Rect, state: &mut EditorState, systems: &mut EditorSystems, response: &egui::Response, layer_drop_idx: &mut Option<(usize, LayerParent)>) -> Option<()> {
        let layer = state.project.layers.get(self.layer)?; 

        let mut set_name = false;
        let mut delete_layer = false;
        let mut show_hide_layer = false; 
        let mut open_close_layer = false;
        let mut set_layer_kind = None;

        let indent_size = 10.0;
        let layer_group_triangle_width = 20.0;

        if self.layer == state.active_layer {
            ui.painter().rect(rect, 0.0, super::HIGHLIGHT, egui::Stroke::NONE);
        }
        if timeline.layer_editing_name == self.layer {
            let text_input = egui::TextEdit::singleline(&mut timeline.layer_edit_curr_name);
            let response = ui.put(rect, text_input);
            if response.lost_focus() {
                set_name = true;
            }
        } else {
            self.render_layer_name(layer, ui, &rect, indent_size, layer_group_triangle_width, &mut open_close_layer);
            self.layer_context_menu(timeline, layer, response, &mut set_layer_kind, &mut delete_layer, systems);
            if response.clicked() && layer.kind == LayerKind::Animation {
                state.active_layer = self.layer;
            }

            self.render_layer_icons(layer, ui, &rect, &mut show_hide_layer);
        }
    
        let showing_layer = layer.show;
        let layer_open = layer.open;
        if let (Some(pointer), Some(payload)) = (
            ui.input(|i| i.pointer.hover_pos()),
            response.dnd_hover_payload::<ObjPtr<Layer>>(),
        ) {
            if rect.contains(pointer) {
                let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
                let mut x_range = rect.x_range();
                x_range.min += (self.indent as f32) * indent_size + layer_group_triangle_width;
                if pointer.y < rect.center().y {
                    ui.painter().hline(x_range, rect.top(), stroke);
                    if *payload.as_ref() != self.layer {
                        *layer_drop_idx = Some((self.local_idx, layer.parent));
                    }
                } else {
                    if layer.kind == LayerKind::Group {
                        let mut x_range = x_range;
                        x_range.min += indent_size;
                        ui.painter().hline(x_range, rect.bottom(), stroke);
                        if *payload.as_ref() != self.layer {
                            *layer_drop_idx = Some((0, LayerParent::Layer(self.layer)));
                        }
                    } else {
                        ui.painter().hline(x_range, rect.bottom(), stroke);
                        if *payload.as_ref() != self.layer {
                            *layer_drop_idx = Some((self.local_idx + 1, layer.parent));
                        }
                    }
                }
            }
        }

        if delete_layer {
            if let Some(act) = Layer::delete(&mut state.project, self.layer) {
                state.actions.add(Action::from_single(act));
            }
        }
        if set_name {
            if let Some(act) = Layer::set_name(&mut state.project, timeline.layer_editing_name, timeline.layer_edit_curr_name.clone()) {
                state.actions.add(Action::from_single(act));
            }
            timeline.layer_editing_name = ObjPtr::null();
        }
        if show_hide_layer {
            if let Some(act) = Layer::set_show(&mut state.project, self.layer, !showing_layer) {
                state.actions.add(Action::from_single(act));
            }
        }
        if open_close_layer {
            if let Some(act) = Layer::set_open(&mut state.project, self.layer, !layer_open) {
                state.actions.add(Action::from_single(act));
            }
        }
        if let Some(layer_kind) = set_layer_kind {
            if let Some(act) = Layer::set_kind(&mut state.project, self.layer, layer_kind) {
                state.actions.add(Action::from_single(act));
            }
        }
    
        None
    }

    fn draggable(&self, timeline: &TimelinePanel) -> bool {
        self.layer != timeline.layer_editing_name
    }

}

fn drop_layer(state: &mut EditorState, layer_ptr: ObjPtr<Layer>, new_idx: usize, new_parent: LayerParent) {
    let mut acts = Vec::new();
    let mut valid = true;
    if let Some(layer) = state.project.layers.get(layer_ptr) {
        if layer.parent != new_parent {
            if let Some(act) = Layer::transfer(&mut state.project, layer_ptr, new_parent) {
                acts.push(act);
            } else {
                valid = false;
            }
        }
    } 
    if valid {
        if let Some(act) = Layer::set_index(&mut state.project, layer_ptr, new_idx) {
            acts.push(act);
        }
        state.actions.add(Action::from_list(acts));
    }
}

pub fn layers(timeline: &mut TimelinePanel, ui: &mut egui::Ui, frame_h: f32, state: &mut EditorState, systems: &mut EditorSystems, grid_rows: &Vec<FrameGridRow>, sidebar_w: f32) {
    let colors = dnd_drop_zone_setup_colors(ui);
    let init_stroke = std::mem::replace(&mut ui.visuals_mut().widgets.active.bg_stroke.color, egui::Color32::TRANSPARENT);
    let mut layer_drop_idx = None;
    if let (_, Some(payload)) = ui.dnd_drop_zone::<ObjPtr<Layer>>(egui::Frame::default().inner_margin(0.0).outer_margin(0.0), |ui| {
        let orig_item_spacing = std::mem::replace(&mut ui.spacing_mut().item_spacing, egui::Vec2::ZERO);
        for row in grid_rows {
            let mut render_layer = |ui: &mut egui::Ui, timeline: &mut TimelinePanel, _: bool| {
                let (rect, response) = ui.allocate_exact_size(Vec2::new(sidebar_w, frame_h), egui::Sense::click());
                row.render_layer(timeline, ui, rect, state, systems, &response, &mut layer_drop_idx);
                ((), response)
            };
            if row.draggable(timeline) {
                draggable_widget(ui, row.layer, |ui, dragged| {
                    render_layer(ui, timeline, dragged)
                });
            } else {
                render_layer(ui, timeline, false); 
            }
        }
        ui.spacing_mut().item_spacing = orig_item_spacing;
    }) {
        if let Some((new_idx, new_parent)) = layer_drop_idx {
            let layer_ptr = *payload.as_ref();
            drop_layer(state, layer_ptr, new_idx, new_parent);
        }
    }
    dnd_drop_zone_reset_colors(ui, colors);
    ui.visuals_mut().widgets.active.bg_stroke.color = init_stroke;
}
