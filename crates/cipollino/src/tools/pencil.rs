
use std::{mem, sync::Arc};

use glam::vec2;

use crate::{editor::state::EditorState, panels::scene::ScenePanel, project::{action::{Action, ObjAction}, frame::Frame, obj::{child_obj::ChildObj, ObjPtr}, stroke::{Stroke, StrokePoint}}, util::curve};

use super::{active_frame, Tool};

pub struct Pencil {
    points: Vec<glam::Vec2>,
    curr_stroke_frame: Option<(ObjPtr<Stroke>, ObjPtr<Frame>)>,
    frame_creation_acts: Vec<ObjAction>,
    stroke_act: Option<ObjAction> 
}

impl Pencil {

    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            curr_stroke_frame: None,
            frame_creation_acts: Vec::new(),
            stroke_act: None
        }
    }

}

impl Pencil {

    fn get_action(&mut self) -> Action {
        let mut action = Action::new();
        let acts = std::mem::replace(&mut self.frame_creation_acts, Vec::new());
        action.add_list(acts);
        action.add(mem::replace(&mut self.stroke_act, None).unwrap());
        action
    }

}

impl Tool for Pencil {

    fn mouse_click(&mut self, mouse_pos: glam::Vec2, state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {
        state.pause();
        let active_frame = active_frame(state);
        if active_frame.is_none() {
            return;
        }
        let (frame, frame_act) = active_frame.unwrap(); 

        self.frame_creation_acts = frame_act;

        let offset = vec2(0.001, 0.0);
        let pts = vec![vec![
            StrokePoint {
                pt: mouse_pos,
                a: mouse_pos - offset,
                b: mouse_pos + offset,
            },
            StrokePoint {
                pt: mouse_pos + offset,
                a: mouse_pos - offset,
                b: mouse_pos + offset,
            }
        ]];

        self.points.push(mouse_pos);

        if let Some((stroke, act)) = Stroke::add(&mut state.project, frame, Stroke {
            frame: frame,
            points: pts,
            color: state.color,
            r: state.stroke_r,
            filled: state.stroke_filled
        }) {
            self.curr_stroke_frame = Some((stroke, frame));
            self.stroke_act = Some(act);
        }

    }

    fn mouse_down(&mut self, mouse_pos: glam::Vec2, state: &mut EditorState, _scene: &mut ScenePanel) {
        state.pause();

        if let Some((stroke, frame)) = self.curr_stroke_frame {
            if self.points.last().map(|prev_pt| (*prev_pt - mouse_pos).length() > 0.001).unwrap_or(true) {
                self.points.push(mouse_pos);

                let mut pts = Vec::new();
                for pt in &self.points {
                    pts.push(pt.x);
                    pts.push(pt.y);
                }

                let mut stroke_points = Vec::new();
                let curve_pts = curve::fit_curve(2, pts.as_slice(), 1.0);
                for i in 0..(curve_pts.len() / (2 * 3)) {
                    let a = glam::vec2(curve_pts[i * 6 + 0], curve_pts[i * 6 + 1]);
                    let p = glam::vec2(curve_pts[i * 6 + 2], curve_pts[i * 6 + 3]);
                    let b = glam::vec2(curve_pts[i * 6 + 4], curve_pts[i * 6 + 5]);
                    stroke_points.push(StrokePoint {
                        pt: p,
                        a,
                        b,
                    });
                }

                if let Some((new_stroke, act)) = Stroke::add(&mut state.project, frame, Stroke {
                    frame: frame,
                    color: state.color,
                    r: state.stroke_r,
                    filled: state.stroke_filled,
                    points: vec![stroke_points]
                }) {
                    mem::replace(&mut self.stroke_act, Some(act)).unwrap().undo(&mut state.project);
                    Stroke::delete(&mut state.project, stroke);
                    self.curr_stroke_frame = Some((new_stroke, frame));
                }
            }
        }
    }

    fn mouse_release(&mut self, _mouse_pos: glam::Vec2, state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {
        state.pause();
        self.reset(state); 
    }

    fn mouse_cursor(&mut self, _mouse_pos: glam::Vec2, _state: &mut EditorState, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> egui::CursorIcon {
        egui::CursorIcon::Crosshair
    }

    fn tool_panel(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        ui.add(egui::Slider::new(&mut state.stroke_r, 1.0..=50.0));
        ui.checkbox(&mut state.stroke_filled, "Filled");
    }

    fn reset(&mut self, state: &mut EditorState) {
        if let Some(_) = self.curr_stroke_frame {
            state.actions.add(self.get_action());
            self.points.clear();
            self.curr_stroke_frame = None;
        }
    }

    fn get_icon(&self) -> &str {
        egui_phosphor::regular::PENCIL
    }

    fn name(&self) -> &str {
        "Pencil"
    }

    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::D)
    }

}
