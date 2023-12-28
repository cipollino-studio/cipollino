
use glam::Vec2;

use crate::{editor::EditorState, panels::scene::OverlayRenderer, util::{curve::sample, geo::segment_intersect}};

use super::Tool;

pub struct Select {

    lasso_pts: Vec<Vec2>

}

impl Select {
    
    pub fn new() -> Self {
        Select {
            lasso_pts: Vec::new()
        }
    }

}

impl Tool for Select {

    fn mouse_click(&mut self, mouse_pos: Vec2, _state: &mut EditorState) {
        self.lasso_pts.clear();
        self.lasso_pts.push(mouse_pos);
    }

    fn mouse_down(&mut self, mouse_pos: Vec2, _state: &mut EditorState) {
        if let Some(last_pt) = self.lasso_pts.last() {
            if (mouse_pos - *last_pt).length() < 0.05 {
                return;
            }
        }
        self.lasso_pts.push(mouse_pos);
    }

    fn mouse_release(&mut self, _mouse_pos: Vec2, state: &mut EditorState, ui: &mut egui::Ui) {

        if !ui.ctx().input(|i| i.modifiers.shift) {
            state.selected_strokes.clear();
        }
        
        if let Some(pt) = self.lasso_pts.first() {
            self.lasso_pts.push(*pt);
            let inside_lasso = |pt: Vec2| {
                let mut cnt = 0;
                if !self.lasso_pts.is_empty() {
                    let first = *self.lasso_pts.first().unwrap();
                    let last = *self.lasso_pts.last().unwrap();
                    if (first - pt).length() < 0.05 || (last - pt).length() < 0.05 {
                        return true;
                    } 
                }
                for pts in self.lasso_pts.windows(2) {
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
                                    let pt = sample(t, p0.data.pt, p0.data.b, p1.data.a, p1.data.pt);
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
        
        self.lasso_pts.clear();
    }

    fn draw_overlay(&mut self, overlay: &mut OverlayRenderer) {
        if !self.lasso_pts.is_empty() {
            for i in 0..(self.lasso_pts.len() - 1) {
                let p0 = self.lasso_pts[i];
                let p1 = self.lasso_pts[i + 1];
                overlay.line(p0, p1, glam::vec4(1.0, 0.0, 0.0, 1.0));
            }
        }
    }

}
