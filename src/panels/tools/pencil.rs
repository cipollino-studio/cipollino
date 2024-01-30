
use std::sync::Arc;

use glam::vec2;

use crate::{util::curve, project::{point::PointData, action::{ObjAction, Action}, stroke::StrokeData}, panels::scene::ScenePanel};

use super::Tool;

pub struct Pencil {
    points: Vec<glam::Vec2>,
    curr_stroke: Option<u64>,
    frame_creation_act: Option<ObjAction>,
    stroke_acts: Vec<ObjAction>,
    frame: u64
}

impl Pencil {

    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            curr_stroke: None,
            frame_creation_act: None,
            stroke_acts: Vec::new(),
            frame: 0
        }
    }

}

impl Pencil {

    fn get_action(&mut self) -> Action {
        let mut action = Action::new();
        if let Some(act) = std::mem::replace(&mut self.frame_creation_act, None) {
            action.add(act);
        }
        for stroke_act in std::mem::replace(&mut self.stroke_acts, Vec::new()) {
            action.add(stroke_act);
        }
        action
    }

}

impl Tool for Pencil {

    fn mouse_click(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {
        let (frame, frame_act) = super::active_frame(state);
        if let Some((stroke_key, stroke_act)) = state.project.add_stroke(StrokeData { frame, color: state.color, r: state.stroke_r, filled: state.stroke_filled }) {
            self.curr_stroke = Some(stroke_key);
            self.points.clear();
            self.points.push(mouse_pos);
            self.frame_creation_act = frame_act;
            self.frame = frame;

            // If the user just taps the mouse, make a lil point
            let offset = vec2(0.001, 0.0);
            self.stroke_acts.push(stroke_act);
            let (_, pt_act) = state.project.add_point(PointData {
                stroke: stroke_key,
                pt: mouse_pos,
                a: mouse_pos - offset,
                b: mouse_pos + offset,
                chain: 0
            }).unwrap();
            self.stroke_acts.push(pt_act);
            let (_, pt_act) = state.project.add_point(PointData {
                stroke: stroke_key,
                pt: mouse_pos + offset,
                a: mouse_pos - offset,
                b: mouse_pos + offset,
                chain: 0
            }).unwrap();
            self.stroke_acts.push(pt_act);

        }
    }

    fn mouse_down(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::EditorState, _scene: &mut ScenePanel) {
        if let Some(stroke) = self.curr_stroke {
            let prev_pt = self.points.last().unwrap();
            if (*prev_pt - mouse_pos).length() > 0.001 {
                self.points.push(mouse_pos);
                if let Some((new_stroke_key, act)) = state.project.add_stroke(StrokeData { frame: self.frame, color: state.color, r: state.stroke_r, filled: state.stroke_filled }) {
                    self.stroke_acts.clear();
                    self.stroke_acts.push(act);

                    state.project.delete_stroke(stroke);
                    let stroke = new_stroke_key;
                    self.curr_stroke = Some(new_stroke_key);

                    let mut pts = Vec::new();
                    for pt in &self.points {
                        pts.push(pt.x);
                        pts.push(pt.y);
                    }

                    let curve_pts = curve::fit_curve(2, pts.as_slice(), 0.01);
                    for i in 0..(curve_pts.len() / (2 * 3)) {
                        let a = glam::vec2(curve_pts[i * 6 + 0], curve_pts[i * 6 + 1]);
                        let p = glam::vec2(curve_pts[i * 6 + 2], curve_pts[i * 6 + 3]);
                        let b = glam::vec2(curve_pts[i * 6 + 4], curve_pts[i * 6 + 5]);
                        if let Some((_key, act)) = state.project.add_point(PointData {
                            pt: p,
                            a,
                            b,
                            stroke,
                            chain: 0
                        }) {
                            self.stroke_acts.push(act);
                        }
                    }
                }
            }
        }
    }

    fn mouse_release(&mut self, _mouse_pos: glam::Vec2, state: &mut crate::editor::EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {
        self.reset(state); 
    }

    fn mouse_cursor(&mut self, _mouse_pos: glam::Vec2, _state: &mut crate::editor::EditorState, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> egui::CursorIcon {
        egui::CursorIcon::Crosshair
    }

    fn tool_panel(&mut self, ui: &mut egui::Ui, state: &mut crate::editor::EditorState) {
        let mut color = [state.color.x, state.color.y, state.color.z];
        ui.color_edit_button_rgb(&mut color);
        state.color = glam::Vec3::from_slice(&color);

        ui.add(egui::Slider::new(&mut state.stroke_r, 0.01..=1.0));

        ui.checkbox(&mut state.stroke_filled, "Filled");
    }

    fn reset(&mut self, state: &mut crate::editor::EditorState) {
        if let Some(_stroke) = self.curr_stroke {
            state.actions.add(self.get_action());
            self.points.clear();
            self.curr_stroke = None;
        }
    }

}
