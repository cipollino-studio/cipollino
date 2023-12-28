
use crate::{util::curve, project::{point::PointData, action::{ObjAction, Action}}};

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

impl Tool for Pencil {

    fn mouse_click(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::EditorState) {
        let (frame, act) = super::active_frame(state);
        if let Some((stroke_key, _act)) = state.project.add_stroke(frame) {
            self.curr_stroke = Some(stroke_key);
            self.points.clear(); 
            self.points.push(mouse_pos);
            self.frame_creation_act = act;
            self.frame = frame;
        }
    }

    fn mouse_down(&mut self, mouse_pos: glam::Vec2, state: &mut crate::editor::EditorState) {
        if let Some(stroke) = self.curr_stroke {
            let prev_pt = self.points.last().unwrap();
            if (*prev_pt - mouse_pos).length() > 0.001 {
                self.points.push(mouse_pos);
                if let Some((new_stroke_key, act)) = state.project.add_stroke(self.frame) {
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

                    if pts.len() > 4 {
                        let curve_pts = curve::fit_curve(2, pts.as_slice(), 0.01);
                        for i in 0..(curve_pts.len() / (2 * 3)) {
                            let a = glam::vec2(curve_pts[i * 6 + 0], curve_pts[i * 6 + 1]);
                            let p = glam::vec2(curve_pts[i * 6 + 2], curve_pts[i * 6 + 3]);
                            let b = glam::vec2(curve_pts[i * 6 + 4], curve_pts[i * 6 + 5]);
                            if let Some((_key, act)) = state.project.add_point(PointData {
                                pt: p,
                                a,
                                b,
                                stroke
                            }) {
                                self.stroke_acts.push(act);
                            }
                        }
                    }
                }
            }
        }
    }

    fn mouse_release(&mut self, _mouse_pos: glam::Vec2, state: &mut crate::editor::EditorState, _ui: &mut egui::Ui) {
        let mut action = Action::new();
        if let Some(act) = std::mem::replace(&mut self.frame_creation_act, None) {
            action.add(act);
        }
        for stroke_act in std::mem::replace(&mut self.stroke_acts, Vec::new()) {
            action.add(stroke_act);
        }
        state.actions.add(action);
    }

    fn mouse_cursor(&mut self, _mouse_pos: glam::Vec2, _state: &mut crate::editor::EditorState) -> egui::CursorIcon {
        egui::CursorIcon::Crosshair
    }

}
