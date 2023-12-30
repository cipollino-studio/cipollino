
use std::sync::Arc;

use glam::{Vec2, vec3, vec2, Mat4};

use crate::{editor::EditorState, panels::scene::{OverlayRenderer, ScenePanel}, util::{curve::{bezier_sample, self}, geo::segment_intersect}, project::{point::PointData, action::Action}};

use super::Tool;

struct Lasso;

impl Lasso {

    fn mouse_click(mouse_pos: Vec2, select: &mut Select) {
        select.lasso_pts.clear();
        select.lasso_pts.push(mouse_pos);
    }

    fn mouse_down(mouse_pos: Vec2, select: &mut Select) {
        if let Some(last_pt) = select.lasso_pts.last() {
            if (mouse_pos - *last_pt).length() < 0.05 {
                return;
            }
        }
        select.lasso_pts.push(mouse_pos);
    }

    fn mouse_release(state: &mut EditorState, select: &mut Select) {

        if let Some(pt) = select.lasso_pts.first() {
            select.lasso_pts.push(*pt);
            let inside_lasso = |pt: Vec2| {
                let mut cnt = 0;
                if !select.lasso_pts.is_empty() {
                    let first = *select.lasso_pts.first().unwrap();
                    let last = *select.lasso_pts.last().unwrap();
                    if (first - pt).length() < 0.05 || (last - pt).length() < 0.05 {
                        return true;
                    } 
                }
                for pts in select.lasso_pts.windows(2) {
                    if let Some(_intersection) = segment_intersect(pt, pt + Vec2::new(1000000.0, 0.0), pts[0], pts[1]) {
                        cnt += 1;
                    }
                }
                cnt % 2 == 1 
            };

            if let Some(gfx) = state.project.graphics.get(&state.open_graphic) {
                for layer in &gfx.layers {
                    if let Some(Some(frame)) = state.project.get_frame_at(*layer, state.frame()).map(|key| state.project.frames.get(&key)) {
                        for stroke_key in &frame.strokes {
                            let stroke = state.project.strokes.get(&stroke_key).unwrap();
                            'pt_loop: for (p0, p1) in stroke.iter_point_pairs() {
                                let p0 = state.project.points.get(&p0).unwrap();
                                let p1 = state.project.points.get(&p1).unwrap();
                                for i in 0..10 {
                                    let t = (i as f32) / 9.0;
                                    let pt = bezier_sample(t, p0.data.pt, p0.data.b, p1.data.a, p1.data.pt);
                                    if inside_lasso(pt) {
                                        state.selected_strokes.push(*stroke_key);
                                        break 'pt_loop;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if state.selected_strokes.len() > 0 {
            select.state = SelectState::FreeTransform;
            select.bb_min = Vec2::INFINITY;
            select.bb_max = -Vec2::INFINITY;
            for stroke_key in &state.selected_strokes {
                if let Some(stroke) = state.project.strokes.get(stroke_key) {
                    for (p0, p1) in stroke.iter_point_pairs() {
                        let p0 = state.project.points.get(&p0).unwrap();
                        let p1 = state.project.points.get(&p1).unwrap();
                        let (min, max) = curve::bezier_bounding_box(p0.data.pt, p0.data.b, p1.data.a, p1.data.pt);
                        select.bb_min = select.bb_min.min(min);
                        select.bb_max = select.bb_max.max(max);
                    }
                }
            }
            select.trans = glam::Mat4::IDENTITY;
        }
        
        select.lasso_pts.clear();
    }

    fn draw_overlay(overlay: &mut OverlayRenderer, select: &mut Select) {
        if !select.lasso_pts.is_empty() {
            for i in 0..(select.lasso_pts.len() - 1) {
                let p0 = select.lasso_pts[i];
                let p1 = select.lasso_pts[i + 1];
                overlay.line(p0, p1, glam::vec4(1.0, 0.0, 0.0, 1.0));
            }
        }
    }

}

struct FreeTransform;

impl FreeTransform {

    fn mouse_click(mouse_pos: Vec2, state: &mut EditorState, ui: &mut egui::Ui, select: &mut Select, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        if let Some(_stroke) = scene.sample_pick(mouse_pos, gl) {
            select.state = SelectState::Translate;
            return;
        }

        select.state = SelectState::Lasso;
        if !ui.input(|i| i.modifiers.shift) {
            state.selected_strokes.clear();
        }
    }

    fn draw_overlay(overlay: &mut OverlayRenderer, select: &mut Select) {
        let bl = select.bb_min;
        let tl = vec2(select.bb_min.x, select.bb_max.y); 
        let tr = select.bb_max;
        let br = vec2(select.bb_max.x, select.bb_min.y);

        let transform = |pt: Vec2| {
            let pt3 = select.trans.transform_point3(vec3(pt.x, pt.y, 0.0));
            vec2(pt3.x, pt3.y)
        };

        let bl_t = transform(bl); 
        let tl_t = transform(tl); 
        let br_t = transform(br); 
        let tr_t = transform(tr); 

        overlay.line(bl_t, tl_t, glam::vec4(1.0, 0.0, 0.0, 1.0));
        overlay.line(tl_t, tr_t, glam::vec4(1.0, 0.0, 0.0, 1.0));
        overlay.line(tr_t, br_t, glam::vec4(1.0, 0.0, 0.0, 1.0));
        overlay.line(br_t, bl_t, glam::vec4(1.0, 0.0, 0.0, 1.0)); 
    }

    fn mouse_cursor(mouse_pos: Vec2, scene: &mut ScenePanel, gl: &Arc<glow::Context>) -> egui::CursorIcon {
        if let Some(_stroke_key) = scene.sample_pick(mouse_pos, gl) {
            return egui::CursorIcon::Move;
        }
        egui::CursorIcon::Default
    }

}

struct Translate;

impl Translate {

    fn mouse_down(mouse_pos: Vec2, state: &mut EditorState, select: &mut Select) {
        let delta = mouse_pos - select.prev_mouse_pos;
        select.apply_transformation(glam::Mat4::from_translation(glam::vec3(delta.x, delta.y, 0.0)), state);
    }

    fn mouse_release(select: &mut Select, state: &mut EditorState) {
        select.state = SelectState::FreeTransform;
        if let Some(action) = std::mem::replace(&mut select.transform_action, None){
            state.actions.add(action);
        } 
    }

}

enum SelectState {
    Lasso,
    FreeTransform,
    Translate
}

pub struct Select {
    state: SelectState,
    lasso_pts: Vec<Vec2>,
    bb_min: Vec2,
    bb_max: Vec2,
    trans: glam::Mat4,
    prev_mouse_pos: Vec2,
    transform_action: Option<Action>
}

impl Select {
    
    pub fn new() -> Self {
        Select {
            state: SelectState::Lasso,
            lasso_pts: Vec::new(),
            bb_min: Vec2::ZERO,
            bb_max: Vec2::ZERO,
            trans: glam::Mat4::IDENTITY,
            prev_mouse_pos: Vec2::ZERO,
            transform_action: None
        }
    }

    pub fn apply_transformation(&mut self, trans: glam::Mat4, state: &mut EditorState) {
        let mut points = Vec::new();
        for stroke in &state.selected_strokes {
            if let Some(stroke) = state.project.strokes.get(stroke) {
                for point_key in &stroke.points {
                    points.push(*point_key);
                }
            }
        }
        let transform_vec2 = |pt: Vec2, mat: Mat4| {
            let v3 = mat.transform_point3(vec3(pt.x, pt.y, 0.0));
            vec2(v3.x, v3.y)
        };

        let trans_inv = self.trans.inverse();
        for point_key in &points {
            let point_key = *point_key;
            if let Some(point) = state.project.points.get(&point_key) {
                let data = point.data.clone();
                state.project.set_point_data(point_key, PointData {
                    a: transform_vec2(data.a, trans_inv),
                    pt: transform_vec2(data.pt, trans_inv),
                    b: transform_vec2(data.b, trans_inv),
                    ..data
                });
            }
        }
        let mut action = Action::new();
        let new_trans = self.trans * trans;
        for point_key in &points {
            let point_key = *point_key;
            if let Some(point) = state.project.points.get(&point_key) {
                let data = point.data.clone();
                if let Some(act) = state.project.set_point_data(point_key, PointData {
                    a: transform_vec2(data.a, new_trans),
                    pt: transform_vec2(data.pt, new_trans),
                    b: transform_vec2(data.b, new_trans),
                    ..data
                }) {
                    action.add(act);
                }
            }
        }
        self.trans = new_trans;

        self.transform_action = Some(action);
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

    fn mouse_down(&mut self, mouse_pos: Vec2, state: &mut EditorState) {
        match self.state {
            SelectState::Lasso => Lasso::mouse_down(mouse_pos, self),
            SelectState::Translate => Translate::mouse_down(mouse_pos, state, self),
            _ => {}
        }
        self.prev_mouse_pos = mouse_pos;
    }

    fn mouse_release(&mut self, _mouse_pos: Vec2, state: &mut EditorState, _ui: &mut egui::Ui) {
        match self.state {
            SelectState::Lasso => Lasso::mouse_release(state, self),
            SelectState::Translate => Translate::mouse_release(self, state),
            _ => {}
        }
    }

    fn mouse_cursor(&mut self, mouse_pos: Vec2, _state: &mut EditorState, scene: &mut ScenePanel, gl: &Arc<glow::Context>) -> egui::CursorIcon {
        match self.state {
            SelectState::FreeTransform => FreeTransform::mouse_cursor(mouse_pos, scene, gl),
            SelectState::Translate => egui::CursorIcon::Move,
            _ => egui::CursorIcon::Default
        }
    }

    fn draw_overlay(&mut self, overlay: &mut OverlayRenderer, _state: &mut EditorState) {
        match self.state {
            SelectState::Lasso => Lasso::draw_overlay(overlay, self), 
            SelectState::FreeTransform | SelectState::Translate => FreeTransform::draw_overlay(overlay, self),
        }
    }

    fn reset(&mut self, state: &mut EditorState) {
        if let Some(action) = std::mem::replace(&mut self.transform_action, None){
            state.actions.add(action);
        } 
        self.state = SelectState::Lasso; 
        self.lasso_pts.clear();
    }

}
