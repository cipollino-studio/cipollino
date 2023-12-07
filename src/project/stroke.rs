
use crate::renderer::mesh::Mesh;

use super::{ObjData, Project, action::ObjAction};

#[derive(Clone)]
pub struct StrokeData {
    frame: u64
}

impl ObjData for StrokeData {

    fn add(&self, key: u64, project: &mut super::Project) {
        project.add_stroke_with_key(key, self.frame);
    }

    fn delete(&self, key: u64, project: &mut super::Project) {
        project.delete_stroke(key);
    }

    fn set(&self, key: u64, project: &mut super::Project) {
        project.set_stroke_data(key, self.clone());
    }
}

pub struct Stroke {
    pub data: StrokeData,
    pub points: Vec<u64>,
    pub mesh: Option<Mesh>,
    pub need_remesh: bool
}

impl Project {

    pub fn add_stroke(&mut self, frame: u64) -> Option<(u64, ObjAction)> {
        let key = self.next_key();
        self.add_stroke_with_key(key, frame)
    }
    
    pub fn add_stroke_with_key(&mut self, key: u64, frame: u64) -> Option<(u64, ObjAction)> {
        self.frames.get_mut(&frame)?.strokes.push(key);
        let data = StrokeData {
            frame,
        };
        self.strokes.insert(key, Stroke {
            data: data.clone(), 
            points: Vec::new(),
            mesh: None,
            need_remesh: true
        });
        Some((key, ObjAction::addition(key, data)))
    }

    pub fn delete_stroke(&mut self, key: u64) -> Option<Vec<ObjAction>> {
        let stroke = self.strokes.remove(&key)?;
        let mut acts = Vec::new();
        for point in stroke.points.iter().rev() {
            if let Some(mut point_acts) = self.delete_point(*point) {
                acts.append(&mut point_acts);
            }
        }
        if let Some(frame) = self.frames.get_mut(&stroke.data.frame) {
            frame.strokes.retain(|stroke| *stroke != key);
        }
        acts.push(ObjAction::deletion(key, stroke.data));
        Some(acts) 
    }
    
    pub fn set_stroke_data(&mut self, key: u64, data: StrokeData) -> Option<ObjAction> {
        let stroke = self.strokes.get_mut(&key)?;
        let res = ObjAction::modification(key, stroke.data.clone(), data.clone());
        stroke.data = data;
        stroke.need_remesh = true;
        Some(res)
    }

}
