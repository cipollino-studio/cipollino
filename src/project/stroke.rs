
use crate::renderer::mesh::Mesh;

use super::{ObjData, Project, action::ObjAction};

fn default_color() -> glam::Vec3 {
    glam::Vec3::ZERO
}

fn default_stroke_r() -> f32 {
    0.05
}

fn default_stroke_filled() -> bool {
    false
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct StrokeData {
    pub frame: u64,
    #[serde(default = "default_color")]
    pub color: glam::Vec3,
    #[serde(default = "default_stroke_r")]
    pub r: f32,
    #[serde(default = "default_stroke_filled")]
    pub filled: bool
}

impl ObjData for StrokeData {

    fn add(&self, key: u64, project: &mut super::Project) {
        project.add_stroke_with_key(key, self.clone());
    }

    fn delete(&self, key: u64, project: &mut super::Project) {
        project.delete_stroke(key);
    }

    fn set(&self, key: u64, project: &mut super::Project) {
        project.set_stroke_data(key, self.clone());
    }
}

// Hack to get around Serde's weird default system
pub fn ret_true() -> bool {
    true
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Stroke {
    pub data: StrokeData,
    pub points: Vec<u64>,
    #[serde(skip)]
    pub mesh: Option<Mesh>,
    #[serde(skip, default = "ret_true")]
    pub need_remesh: bool
}

impl Stroke {

    pub fn iter_point_pairs(&self) -> impl Iterator<Item = (u64, u64)> + '_ {
        self.points.windows(2).map(|arr| (arr[0], arr[1]))
    }

}

impl Project {

    pub fn add_stroke(&mut self, data: StrokeData) -> Option<(u64, ObjAction)> {
        let key = self.next_key();
        self.add_stroke_with_key(key, data)
    }
    
    pub fn add_stroke_with_key(&mut self, key: u64, data: StrokeData) -> Option<(u64, ObjAction)> {
        self.frames.get_mut(&data.frame)?.strokes.push(key);
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
