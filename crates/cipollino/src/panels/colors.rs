
use crate::{editor::EditorState, project::{action::{Action, ObjAction}, obj::child_obj::ChildObj, palette::PaletteColor, stroke::StrokeColor}, util::ui::color::color_picker};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ColorPanel {
    #[serde(skip)]
    curr_act: Option<ObjAction>
}

impl ColorPanel {

    pub fn new() -> Self {
        ColorPanel {
            curr_act: None
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if state.project.palettes.get(state.open_palette).is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No palette open");
            });
            return; 
        };

        egui::TopBottomPanel::top(ui.next_auto_id()).show_inside(ui, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button(egui_phosphor::regular::PLUS).clicked() {
                    if let Some((_ptr, act)) = PaletteColor::add(&mut state.project, state.open_palette, PaletteColor {
                        palette: state.open_palette,
                        ..Default::default() 
                    }) {
                        state.actions.add(Action::from_single(act));
                    }
                }
            });
        });

        let mut edit_color = None;
        let mut set_color = false;
        let palette = state.project.palettes.get(state.open_palette).unwrap();
        egui::CentralPanel::default().show_inside(ui, |ui| {
            for color_box in &palette.colors {
            let color = color_box.get(&state.project); 
                let mut col = color.color;
                ui.horizontal(|ui| {
                    let mut editing_color = false;
                    color_picker(ui, &mut col, None, &mut editing_color, &mut set_color);
                    if editing_color {
                        edit_color = Some((color_box.make_ptr(), col));
                    }

                    if ui.button(egui_phosphor::regular::EYEDROPPER_SAMPLE).clicked() {
                        state.color = StrokeColor::Palette(color_box.make_ptr(), color.color);
                    }
                });
                
            }
        });
        
        if let Some((ptr, color)) = edit_color {
            if let Some(prev_act) = std::mem::replace(&mut self.curr_act, None) {
                prev_act.undo(&mut state.project);
            }
            if let Some(act) = PaletteColor::set_color(&mut state.project, ptr, color) {
                self.curr_act = Some(act);
            }
        }
        if set_color {
            if let Some(act) = std::mem::replace(&mut self.curr_act, None) {
                state.actions.add(Action::from_single(act));
            }
        }
        
    }

}
