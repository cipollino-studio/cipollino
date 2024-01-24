
use std::sync::Arc;

use glam::{Vec2, vec3, vec2, Mat4, Quat};

use crate::{editor::EditorState, panels::scene::{OverlayRenderer, ScenePanel}, util::{curve::{bezier_sample, self}, geo::segment_intersect}, project::{point::PointData, action::Action}};

use super::Tool;

struct Lasso;

impl Lasso {

    fn mouse_click(mouse_pos: Vec2, select: &mut Select) {
        select.lasso_pts.clear();
        select.lasso_pts.push(mouse_pos);
    }

    fn mouse_down(mouse_pos: Vec2, select: &mut Select, scene: &mut ScenePanel) {
        if let Some(last_pt) = select.lasso_pts.last() {
            if (mouse_pos - *last_pt).length() < 0.005 * scene.cam_size {
                return;
            }
        }
        select.lasso_pts.push(mouse_pos);
    }

    fn mouse_release(state: &mut EditorState, select: &mut Select, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {

        if select.lasso_pts.len() == 1 {
            if let Some(stroke_key) = scene.sample_pick(select.lasso_pts[0], gl) {
                state.selected_strokes.push(stroke_key);
            }
        } else if let Some(pt) = select.lasso_pts.first() {
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
            select.pivot = (select.bb_max + select.bb_min) * 0.5;
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
        let FreeTransformPoints {
            bl: _,
            tl: _,
            br: _,
            tr: _,
            bl_rotate,
            tl_rotate,
            br_rotate,
            tr_rotate
        } = select.freetransform_points(scene.cam_size);
        let r = Self::overlay_circle_r(scene.cam_size);

        if (mouse_pos - tr_rotate).length() < r || (mouse_pos - tl_rotate).length() < r || (mouse_pos - br_rotate).length() < r || (mouse_pos - bl_rotate).length() < r {
            select.state = SelectState::Rotate;
            return;
        }

        if let Some(stroke) = scene.sample_pick(mouse_pos, gl) {
            if state.selected_strokes.contains(&stroke) {
                select.state = SelectState::Translate;
            } else {
                select.state = SelectState::Lasso;
                if !ui.input(|i| i.modifiers.shift) {
                    state.selected_strokes.clear();
                }
            }
            return;
        }

        select.state = SelectState::Lasso;
        if !ui.input(|i| i.modifiers.shift) {
            state.selected_strokes.clear();
        }
    }

    fn overlay_circle_r(cam_size: f32) -> f32 {
        0.025 * cam_size
    }

    fn draw_overlay(overlay: &mut OverlayRenderer, select: &mut Select) {
        let FreeTransformPoints {
            bl,
            tl,
            br,
            tr,
            bl_rotate,
            tl_rotate,
            br_rotate,
            tr_rotate
        } = select.freetransform_points(overlay.cam_size);
        
        let color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        let r = Self::overlay_circle_r(overlay.cam_size);

        overlay.line(bl, tl, color);
        overlay.line(tl, tr, color);
        overlay.line(tr, br, color);
        overlay.line(br, bl, color);
        overlay.circle(select.transform(select.pivot), color, r);

        overlay.circle(tr_rotate, color, r);
        overlay.circle(tl_rotate, color, r);
        overlay.circle(br_rotate, color, r);
        overlay.circle(bl_rotate, color, r);
        
    }

    fn mouse_cursor(mouse_pos: Vec2, scene: &mut ScenePanel, gl: &Arc<glow::Context>, select: &mut Select) -> egui::CursorIcon {
        let FreeTransformPoints {
            bl: _,
            tl: _,
            br: _,
            tr: _,
            bl_rotate,
            tl_rotate,
            br_rotate,
            tr_rotate
        } = select.freetransform_points(scene.cam_size);
        let r = Self::overlay_circle_r(scene.cam_size);

        if (mouse_pos - tr_rotate).length() < r || (mouse_pos - tl_rotate).length() < r || (mouse_pos - br_rotate).length() < r || (mouse_pos - bl_rotate).length() < r {
            return egui::CursorIcon::Alias;
        }
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
        let (scl, rot, trans) = select.trans.to_scale_rotation_translation();
        select.apply_transformation(Mat4::from_scale_rotation_translation(scl, rot, trans + glam::vec3(delta.x, delta.y, 0.0)), state);
    }

    fn mouse_release(select: &mut Select, state: &mut EditorState) {
        select.state = SelectState::FreeTransform;
        if let Some(action) = std::mem::replace(&mut select.transform_action, None){
            state.actions.add(action);
        } 
    }

}

struct Rotate;

impl Rotate {

    fn mouse_down(mouse_pos: Vec2, state: &mut EditorState, select: &mut Select) {
        let pivot = select.transform(select.pivot);
        let angle = -(mouse_pos - pivot).angle_between(select.prev_mouse_pos - pivot);
        let (scl, rot, trans) = select.trans.to_scale_rotation_translation();
        let new_angle = rot.to_euler(glam::EulerRot::XYZ).2 + angle;
        let new_rot = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, new_angle);
        let new_trans_unpivoted = Mat4::from_scale_rotation_translation(scl, new_rot, trans);
        let new_pivot = new_trans_unpivoted.transform_point3(vec3(select.pivot.x, select.pivot.y, 0.0));
        let new_pivot = vec2(new_pivot.x, new_pivot.y);
        let delta = pivot - new_pivot;
        let new_trans = Mat4::from_scale_rotation_translation(scl, new_rot, trans + vec3(delta.x, delta.y, 0.0));
        select.apply_transformation(new_trans, state);
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
    Translate,
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
            transform_action: None
        }
    }

    pub fn apply_transformation(&mut self, new_trans: glam::Mat4, state: &mut EditorState) {
        let mut points = Vec::new();
        for stroke in &state.selected_strokes {
            if let Some(stroke) = state.project.strokes.get(stroke) {
                for chain in &stroke.points {
                    for point_key in chain {
                        points.push(*point_key);
                    }
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

    pub fn transform(&self, pt: Vec2) -> Vec2 {
        let pt3 = self.trans.transform_point3(vec3(pt.x, pt.y, 0.0));
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
            SelectState::Rotate => Rotate::mouse_down(mouse_pos, state, self),
            _ => {}
        }
        self.prev_mouse_pos = mouse_pos;
    }

    fn mouse_release(&mut self, _mouse_pos: Vec2, state: &mut EditorState, _ui: &mut egui::Ui, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        match self.state {
            SelectState::Lasso => Lasso::mouse_release(state, self, scene, gl),
            SelectState::Translate => Translate::mouse_release(self, state),
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

    fn draw_overlay(&mut self, overlay: &mut OverlayRenderer, _state: &mut EditorState) {
        match self.state {
            SelectState::Lasso => Lasso::draw_overlay(overlay, self), 
            SelectState::FreeTransform | SelectState::Translate | SelectState::Rotate => FreeTransform::draw_overlay(overlay, self),
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
