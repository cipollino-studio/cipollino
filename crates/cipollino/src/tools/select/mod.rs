
use std::sync::Arc;

use glam::{vec2, vec3, Mat4, Vec2};

use crate::{editor::{selection::Selection, state::EditorState, EditorSystems}, keybind, panels::scene::{overlay::OverlayRenderer, ScenePanel}, project::{action::Action, obj::obj_list::ObjListTrait, stroke::Stroke}};


use self::scale::ScalePivot;

use super::Tool;

mod lasso;
use lasso::Lasso;

mod free_transform;
use free_transform::FreeTransform;

mod translate;
use translate::Translate;

mod scale;
use scale::Scale;

mod rotate;
use rotate::Rotate;

enum SelectState {
    Lasso,
    FreeTransform,
    Translate,
    Scale(ScalePivot),
    Rotate
}

pub struct Select {
    state: SelectState,
    lasso_pts: Vec<Vec2>,
    bb_min: Vec2,
    bb_max: Vec2,
    pivot: Vec2,
    trans: glam::Mat4,
    prev_mouse_pos: Vec2,
    transform_action: Option<Action>
}

struct FreeTransformPoints {
    bl: Vec2,
    tl: Vec2,
    br: Vec2,
    tr: Vec2,
    bl_rotate: Vec2,
    tl_rotate: Vec2,
    br_rotate: Vec2,
    tr_rotate: Vec2,
}

impl Select {
    
    pub fn new() -> Self {
        Select {
            state: SelectState::Lasso,
            lasso_pts: Vec::new(),
            bb_min: Vec2::ZERO,
            bb_max: Vec2::ZERO,
            pivot: Vec2::ZERO,
            trans: glam::Mat4::IDENTITY,
            prev_mouse_pos: Vec2::ZERO,
            transform_action: None,
        }
    }

    pub fn apply_transformation(&mut self, new_trans: glam::Mat4, state: &mut EditorState) {
        if let Selection::Scene(strokes) = &state.selection { 
            let trans_inv = self.trans.inverse();
            for stroke in strokes {
                Stroke::transform(&mut state.project, *stroke, trans_inv);
            }
            
            let mut action = Action::new();
            for stroke in strokes {
                if let Some(act) = Stroke::transform(&mut state.project, *stroke, new_trans) {
                    action.add(act);
                }
            }
            
            self.trans = new_trans;
            self.transform_action = Some(action);
        }
    }

    pub fn transform(&self, pt: Vec2) -> Vec2 {
        let pt3 = self.trans.transform_point3(vec3(pt.x, pt.y, 0.0));
        vec2(pt3.x, pt3.y)
    }

    pub fn untransform(&self, pt: Vec2) -> Vec2 {
        let pt3 = self.trans.inverse().transform_point3(vec3(pt.x, pt.y, 0.0));
        vec2(pt3.x, pt3.y)
    }

    fn freetransform_points(&self, cam_size: f32) -> FreeTransformPoints {
        let bl = self.bb_min;
        let tl = vec2(self.bb_min.x, self.bb_max.y); 
        let tr = self.bb_max;
        let br = vec2(self.bb_max.x, self.bb_min.y);

        let bl_t = self.transform(bl); 
        let tl_t = self.transform(tl); 
        let br_t = self.transform(br); 
        let tr_t = self.transform(tr); 

        let diag = ((tl_t - bl_t).normalize() + (tr_t - tl_t).normalize()).normalize() * 0.06 * cam_size;
        let diag_rot = ((tl_t - bl_t).normalize() - (tr_t - tl_t).normalize()).normalize() * 0.06 * cam_size;

        FreeTransformPoints {
            bl: bl_t,
            tl: tl_t,
            br: br_t,
            tr: tr_t,
            tr_rotate: tr_t + diag,
            tl_rotate: tl_t + diag_rot,
            br_rotate: br_t - diag_rot,
            bl_rotate: bl_t - diag
        }
    }

    pub fn pivot_matrix(&self, mat: glam::Mat4, pivot: glam::Vec2) -> Mat4 {
        let new_pivot = mat.transform_point3(vec3(pivot.x, pivot.y, 0.0));
        let new_pivot = vec2(new_pivot.x, new_pivot.y);
        let delta = self.transform(pivot) - new_pivot;
        let (scl, rot, trans) = mat.to_scale_rotation_translation();
        let new_mat = Mat4::from_scale_rotation_translation(scl, rot, trans + vec3(delta.x, delta.y, 0.0));
        new_mat 
    }

}

impl Tool for Select {

    fn mouse_click(&mut self, mouse_pos: Vec2, state: &mut EditorState, ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        match self.state {
            SelectState::Lasso => Lasso::mouse_click(mouse_pos, self),
            SelectState::FreeTransform => FreeTransform::mouse_click(mouse_pos, state, ui, self, scene, gl),
            _ => {}
        }
        self.prev_mouse_pos = mouse_pos;
    }

    fn mouse_down(&mut self, mouse_pos: Vec2, state: &mut EditorState, scene: &mut ScenePanel) {
        match self.state {
            SelectState::Lasso => Lasso::mouse_down(mouse_pos, self, scene),
            SelectState::Translate => Translate::mouse_down(mouse_pos, state, self),
            SelectState::Scale(pivot) => Scale::mouse_down(mouse_pos, state, self, &scene, pivot),
            SelectState::Rotate => Rotate::mouse_down(mouse_pos, state, self),
            _ => {}
        }
        self.prev_mouse_pos = mouse_pos;
    }

    fn mouse_release(&mut self, _mouse_pos: Vec2, state: &mut EditorState, _ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        match self.state {
            SelectState::Lasso => Lasso::mouse_release(state, self, scene, gl),
            SelectState::Translate => Translate::mouse_release(self, state),
            SelectState::Scale(_) => Scale::mouse_release(self, state),
            SelectState::Rotate => Rotate::mouse_release(self, state),
            _ => {}
        }
    }

    fn mouse_cursor(&mut self, mouse_pos: Vec2, _state: &mut EditorState, scene: &mut ScenePanel, gl: &Arc<glow::Context>) -> egui::CursorIcon {
        match self.state {
            SelectState::FreeTransform => FreeTransform::mouse_cursor(mouse_pos, scene, gl, self),
            SelectState::Translate => egui::CursorIcon::Move,
            SelectState::Rotate => egui::CursorIcon::Alias,
            _ => egui::CursorIcon::Default
        }
    }

    fn draw_overlay(&mut self, overlay: &mut OverlayRenderer, _state: &EditorState) {
        match self.state {
            SelectState::Lasso => Lasso::draw_overlay(overlay, self), 
            SelectState::FreeTransform | SelectState::Translate | SelectState::Scale(_) | SelectState::Rotate => FreeTransform::draw_overlay(overlay, self),
        }
    }

    fn reset(&mut self, state: &mut EditorState) {
        if let Selection::Scene(strokes) = &state.selection {
            self.state = SelectState::FreeTransform;
            self.bb_min = Vec2::INFINITY;
            self.bb_max = -Vec2::INFINITY;
            for stroke_ptr in strokes {
                state.project.strokes.get_then(*stroke_ptr, |stroke| {
                    for segment in stroke.iter_bezier_segments() {
                        let (min, max) = segment.bounding_box();
                        self.bb_min = self.bb_min.min(min);
                        self.bb_max = self.bb_max.max(max);
                    }
                });
            }
            self.pivot = (self.bb_max + self.bb_min) * 0.5;
            self.trans = glam::Mat4::IDENTITY;
        } else {
            if let Some(action) = std::mem::replace(&mut self.transform_action, None) {
                state.actions.add(action);
            } 
            self.state = SelectState::Lasso; 
            self.lasso_pts.clear();
        }
    }

    fn get_icon(&self) -> &str {
        egui_phosphor::regular::CURSOR
    }

    fn name(&self) -> &str {
        "Select"
    }

    fn shortcut(&self, systems: &mut EditorSystems) -> egui::KeyboardShortcut {
        systems.prefs.get::<SelectToolKeybind>()
    }

}

keybind!(SelectToolKeybind, "Select", NONE, V);
