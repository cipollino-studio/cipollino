
use std::sync::Arc;

use glam::Vec2;

use crate::{editor::state::EditorState, panels::scene::{overlay::OverlayRenderer, ScenePanel}, project::{action::{Action, ObjAction}, obj::{child_obj::ChildObj, obj_list::ObjListTrait, ObjPtr}, stroke::{Stroke, StrokePoint}, Project}, tools::state_machine::ToolState, util::geo::LineSegment};

use super::neutral::Neutral;

pub struct Cut {
    pts: Vec<Vec2>
}

impl Cut {

    pub fn new(init_pt: Vec2) -> Self {
        Self {
            pts: vec![init_pt] 
        }
    }

    fn cut_unfilled_stroke(&self, project: &mut Project, stroke_ptr: ObjPtr<Stroke>, acts: &mut Vec<ObjAction>) -> Option<()> {
        let stroke = project.strokes.get(stroke_ptr)?;
        if stroke.filled {
            return None;
        }
        let mut new_strokes = vec![vec![]];
        let frame = stroke.frame;
        let r = stroke.r;
        let color = stroke.color;
        for bezier in stroke.iter_bezier_segments() {
            new_strokes.last_mut().unwrap().push(bezier);
            for line in self.pts.windows(2) {
                let line = LineSegment::new(line[0], line[1]);
                let mut intersection_ts = bezier.intersect_segment_ts(&line);
                if intersection_ts.is_empty() {
                    continue;
                }
                intersection_ts.sort_by(|a, b| a.total_cmp(b));

                let intersections = intersection_ts.iter().map(|t| bezier.sample(*t));
                for intersection in intersections {
                    let curve = new_strokes.last_mut().unwrap().pop().unwrap();
                    let t = curve.nearest_t(intersection);
                    let (before, after) = curve.split(t);
                    new_strokes.last_mut().unwrap().push(before);
                    new_strokes.push(vec![after]);
                } 
            }
        }

        if new_strokes.len() > 1 {
            if let Some(act) = Stroke::delete(project, stroke_ptr) {
                acts.push(act);
            }

            for new_stroke in new_strokes {
                let mut pts = Vec::new(); 
                pts.push(StrokePoint {
                    a: new_stroke[0].p0 - (new_stroke[0].a1 - new_stroke[0].p0),
                    pt: new_stroke[0].p0,
                    b: new_stroke[0].b0
                });
                for bz_pair in new_stroke.windows(2) {
                    let first = bz_pair[0];
                    let next = bz_pair[1]; 
                    pts.push(StrokePoint {
                        a: first.a1,
                        pt: first.p1,
                        b: next.b0
                    });
                }
                let last_curve = new_stroke.last().unwrap();
                pts.push(StrokePoint {
                    a: last_curve.a1, 
                    pt: last_curve.p1,
                    b: last_curve.p1 + (last_curve.p1 - last_curve.a1) 
                });

                if let Some((_, act)) = Stroke::add(project, frame, Stroke {
                    frame,
                    color,
                    r,
                    filled: false,
                    points: vec![pts],
                }) {
                    acts.push(act);
                }
            }
        }

        Some(())
    }

} 

impl ToolState for Cut {

    fn mouse_down(&mut self, mouse_pos: Vec2, _state: &mut EditorState, _scene: &mut ScenePanel) -> Option<Box<dyn ToolState>> {
        let last_pt = *self.pts.last().unwrap();
        if (mouse_pos - last_pt).length() > 0.5 {
            self.pts.push(mouse_pos);
        }
        None
    }

    fn mouse_release(&mut self, _mouse_pos: Vec2, state: &mut EditorState, _ui: &mut egui::Ui, _scene: &mut ScenePanel, _gl: &Arc<glow::Context>) -> Option<Box<dyn ToolState>> {

        let mut acts = Vec::new();

        for stroke_ptr in state.visible_strokes() {
            self.cut_unfilled_stroke(&mut state.project, stroke_ptr, &mut acts); 
        }
        
        if !acts.is_empty() {
            state.actions.add(Action::from_list(acts));
        }

        Some(Box::new(Neutral {})) 
    }

    fn draw_overlay(&mut self, overlay: &mut OverlayRenderer, _state: &EditorState) {
        for line in self.pts.windows(2) {
            let a = line[0];
            let b = line[1];
            overlay.line(a, b, glam::vec4(0.0, 1.0, 1.0, 1.0));
        } 
    }

}