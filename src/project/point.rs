
use glam::Vec2;

use super::{ObjData, Project, action::ObjAction};

#[derive(Clone)]
pub struct PointData {
    pub pt: Vec2,
    pub a: Vec2,
    pub b: Vec2,
    pub stroke: u64
}

impl ObjData for PointData {

    fn add(&self, key: u64, project: &mut super::Project) {
        project.add_point_with_key(key, self.clone());
    }

    fn delete(&self, key: u64, project: &mut super::Project) {
        project.delete_point(key);
    }

    fn set(&self, key: u64, project: &mut super::Project) {
        project.set_point_data(key, self.clone());
    }
}

pub struct Point {
    pub data: PointData 
}

impl Project {

    pub fn add_point(&mut self, data: PointData) -> Option<(u64, ObjAction)> {
        let key = self.next_key();
        self.add_point_with_key(key, data)
    }
    
    pub fn add_point_with_key(&mut self, key: u64, data: PointData) -> Option<(u64, ObjAction)> {
        let stroke = self.strokes.get_mut(&data.stroke)?;
        stroke.points.push(key);
        stroke.need_remesh = true;
        
        self.points.insert(key, Point {
            data: data.clone(), 
        });
        Some((key, ObjAction::addition(key, data)))
    }

    pub fn delete_point(&mut self, key: u64) -> Option<Vec<ObjAction>> {
        let point = self.points.remove(&key)?;
        if let Some(stroke) = self.strokes.get_mut(&point.data.stroke) {
            stroke.points.retain(|point| *point != key);
            stroke.need_remesh = true;
        }
        Some(vec![ObjAction::deletion(key, point.data)]) 
    }
    
    pub fn set_point_data(&mut self, key: u64, data: PointData) -> Option<ObjAction> {
        let point = self.points.get_mut(&key)?;
        let res = ObjAction::modification(key, point.data.clone(), data.clone());
        let stroke_key = point.data.stroke;
        point.data = data;
        self.strokes.get_mut(&stroke_key)?.need_remesh = true;
        Some(res)
    }

}
