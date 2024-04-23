
use unique_type_id::UniqueTypeId;

use crate::{editor::{dialog::Dialog, state::EditorState, EditorSystems}, project::{action::{Action, ObjAction}, layer::{BlendingMode, Layer, LayerKind}, obj::{obj_list::ObjListTrait, ObjPtr}}, util::ui::drag_value};

#[derive(UniqueTypeId)]
pub struct LayerPropertyDialog {
    layer: ObjPtr<Layer>,

    set_alpha_action: Option<ObjAction> 
}

impl LayerPropertyDialog {

    pub fn new(layer: ObjPtr<Layer>) -> Self {
        Self {
            layer,
            set_alpha_action: None
        }
    }

}

impl Dialog for LayerPropertyDialog {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, _systems: &mut EditorSystems) -> bool {
        let layer = if let Some(layer) = state.project.layers.get(self.layer) {
            layer
        } else {
            return true;
        };

        let mut alpha = layer.alpha * 100.0;
        let mut set_alpha = (false, false);
        let mut blending = layer.blending;
        let initial_blending = blending;

        let right_align_layout = egui::Layout::top_down(egui::Align::RIGHT);
        egui::Grid::new(ui.next_auto_id()).min_col_width(80.0).show(ui, |ui| {
            if layer.kind == LayerKind::Animation || layer.kind == LayerKind::Group {
                ui.with_layout(right_align_layout, |ui| {
                    ui.label("Layer Alpha:");
                });
                drag_value(ui, "", &mut alpha, 0.0..=100.0, Some(&mut set_alpha));
                ui.end_row();

                ui.with_layout(right_align_layout, |ui| {
                    ui.label("Blending:");
                });
                egui::ComboBox::new(ui.next_auto_id(), "")
                    .selected_text(blending.name()).show_ui(ui, |ui| {
                        let mut blending_mode_option = |ui: &mut egui::Ui, mode: BlendingMode| {
                            ui.selectable_value(&mut blending, mode, mode.name());
                        };

                        blending_mode_option(ui, BlendingMode::Normal);
                        ui.separator();
                        blending_mode_option(ui, BlendingMode::Add);
                        blending_mode_option(ui, BlendingMode::Screen);
                        blending_mode_option(ui, BlendingMode::ColorDodge);
                        ui.separator();
                        blending_mode_option(ui, BlendingMode::Multiply);
                        blending_mode_option(ui, BlendingMode::ColorBurn);
                        ui.separator();
                        blending_mode_option(ui, BlendingMode::Overlay);
                        blending_mode_option(ui, BlendingMode::SoftLight);
                        blending_mode_option(ui, BlendingMode::HardLight);
                        blending_mode_option(ui, BlendingMode::VividLight);
                        ui.separator();
                        blending_mode_option(ui, BlendingMode::Color);
                });
                ui.end_row();

            }
        });

        let (edit_alpha, set_alpha) = set_alpha;
        if edit_alpha {
            if let Some(act) = std::mem::replace(&mut self.set_alpha_action, None) {
                act.undo(&mut state.project);
            }
            self.set_alpha_action = Layer::set_alpha(&mut state.project, self.layer, alpha / 100.0); 
        } 
        if set_alpha {
            if let Some(act) = std::mem::replace(&mut self.set_alpha_action, None) {
                state.actions.add(Action::from_single(act));
            }
        }

        if blending != initial_blending {
            if let Some(act) = Layer::set_blending(&mut state.project, self.layer, blending) {
                state.actions.add(Action::from_single(act));
            }
        } 

        false
    }

    fn title(&self, _state: &EditorState) -> String {
        "Layer Properties".to_owned()
    }

    fn resizable(&self) -> bool {
        false
    }

}
