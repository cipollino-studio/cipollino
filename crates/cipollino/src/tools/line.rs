

use std::{mem, sync::Arc};

use glam::{vec2, Vec2};

use crate::{editor::state::EditorState, panels::scene::ScenePanel, project::{action::{Action, ObjAction}, frame::Frame, obj::{child_obj::ChildObj, ObjPtr}, stroke::{Stroke, StrokePoint}}};

use super::{active_frame, Tool};

pub struct Line {
    first_point: Vec2,
    curr_stroke_frame: Option<(ObjPtr<Stroke>, ObjPtr<Frame>)>,
    frame_creation_acts: Vec<ObjAction>,
    stroke_act: Option<ObjAction> 
}

impl Line {

    pub fn new() -> Self {
        Self {
            first_point: Vec2::ZERO,
            curr_stroke_frame: None,
            frame_creation_acts: Vec::new(),
            stroke_act: None
        }
    }

}

impl Line {

    fn get_action(&mut self) -> Action {
        let mut action = Action::new();
        let acts = std::mem::replace(&mut self.frame_creation_acts, Vec::new());
        action.add_list(acts);
        action.add(mem::replace(&mut self.stroke_act, None).unwrap());
        action
    }

}

impl Tool for Line {

    fn mouse_click(&mut self, mouse_pos: glam::Vec2, state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) {
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

        self.first_point = mouse_pos;

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
        if let Some((stroke, frame)) = self.curr_stroke_frame {
            let dir = (mouse_pos - self.first_point) / 3.0;
            if let Some((new_stroke, act)) = Stroke::add(&mut state.project, frame, Stroke {
                frame: frame,
                color: state.color,
                r: state.stroke_r,
                filled: state.stroke_filled,
                points: vec![vec![
                    StrokePoint { a: self.first_point - dir, pt: self.first_point, b: self.first_point + dir },
                    StrokePoint { a: mouse_pos - dir, pt: mouse_pos, b: mouse_pos + dir }
                ]]
            }) {
                mem::replace(&mut self.stroke_act, Some(act)).unwrap().undo(&mut state.project);
                Stroke::delete(&mut state.project, stroke);
                self.curr_stroke_frame = Some((new_stroke, frame));
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

    fn reset(&mut self, state: &mut EditorState) {
        if let Some(_) = self.curr_stroke_frame {
            state.actions.add(self.get_action());
            self.curr_stroke_frame = None;
        }
    }

    fn get_icon(&self) -> &str {
        egui_phosphor::regular::LINE_SEGMENT
    }

    fn name(&self) -> &str {
        "Line"
    }

    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::L)
    }

}
