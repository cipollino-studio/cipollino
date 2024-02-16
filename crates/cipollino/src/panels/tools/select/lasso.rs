use std::sync::Arc;

use glam::Vec2;

use crate::{editor::EditorState, panels::{scene::{OverlayRenderer, ScenePanel}, tools::Tool}, util::{curve::bezier_sample, geo::segment_intersect}};

use super::Select;


pub struct Lasso;

impl Lasso {

    pub fn mouse_click(mouse_pos: Vec2, select: &mut Select) {
        select.lasso_pts.clear();
        select.lasso_pts.push(mouse_pos);
    }

    pub fn mouse_down(mouse_pos: Vec2, select: &mut Select, scene: &mut ScenePanel) {
        if let Some(last_pt) = select.lasso_pts.last() {
            if (mouse_pos - *last_pt).length() < 0.005 * scene.cam_size {
                return;
            }
        }
        select.lasso_pts.push(mouse_pos);
    }

    pub fn mouse_release(state: &mut EditorState, select: &mut Select, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {

        if select.lasso_pts.len() == 1 {
            if let Some(stroke_key) = scene.sample_pick(select.lasso_pts[0], gl) {
                state.selection.select_stroke(stroke_key);
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

            for stroke_ptr in state.visible_strokes() {
                let stroke = state.project.strokes.get(stroke_ptr);
                if stroke.is_none() {
                    continue;
                } 
                let stroke = stroke.unwrap();
                'pt_loop: for (p0, p1) in stroke.iter_point_pairs() {
                    for i in 0..10 {
                        let t = (i as f32) / 9.0;
                        let pt = bezier_sample(t, p0.pt, p0.b, p1.a, p1.pt);
                        if inside_lasso(pt) {
                            state.selection.select_stroke(stroke_ptr);
                            break 'pt_loop;
                        }
                    }
                }
            }
        }

        select.reset(state);

        select.lasso_pts.clear();
    }

    pub fn draw_overlay(overlay: &mut OverlayRenderer, select: &mut Select) {
        if !select.lasso_pts.is_empty() {
            for i in 0..(select.lasso_pts.len() - 1) {
                let p0 = select.lasso_pts[i];
                let p1 = select.lasso_pts[i + 1];
                overlay.line(p0, p1, glam::vec4(1.0, 0.0, 0.0, 1.0));
            }
        }
    }

}

