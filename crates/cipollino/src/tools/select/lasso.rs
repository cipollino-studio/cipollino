use std::sync::Arc;

use glam::Vec2;

use crate::{editor::state::EditorState, panels::scene::{overlay::OverlayRenderer, ScenePanel}, util::geo::LineSegment};
use super::Select;
use crate::tools::Tool;

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
        state.pause();

        if select.lasso_pts.len() == 1 {
            if let Some(stroke_key) = scene.sample_pick(select.lasso_pts[0], gl) {
                state.selection.select_stroke_inverting(stroke_key);
            }
        } else if let Some(pt) = select.lasso_pts.first() {
            select.lasso_pts.push(*pt);
            let inside_lasso = |pt: Vec2| {
                let mut cnt = 0;
                for pts in select.lasso_pts.windows(2) {
                    if let Some(_intersection) = LineSegment::new(pts[0], pts[1]).intersect(LineSegment::new(pt, pt + Vec2::new(1000000.0, 0.0))) {
                        cnt += 1;
                    }
                }
                cnt % 2 == 1 
            };

            'stroke_loop: for stroke_ptr in state.visible_strokes() {
                let stroke = state.project.strokes.get(stroke_ptr);
                if stroke.is_none() {
                    continue;
                } 
                let stroke = stroke.unwrap();

                macro_rules! select {
                    () => {
                        state.selection.select_stroke_inverting(stroke_ptr); 
                        continue 'stroke_loop;
                    };
                }

                for chain in &stroke.points {
                    if let Some(pt) = chain.first() {
                        if inside_lasso(pt.pt) {
                            select!();
                        }
                    }
                    if let Some(pt) = chain.last() {
                        if inside_lasso(pt.pt) {
                            select!();
                        }
                    } 
                }
                
                for bezier in stroke.iter_bezier_segments() {
                    for pts in select.lasso_pts.windows(2) {
                        let segment = LineSegment::new(pts[0], pts[1]);
                        let intersections = bezier.intersect_segment(&segment);
                        if !intersections.is_empty() {
                            select!();
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

