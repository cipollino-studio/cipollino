
use project_macros::{ObjClone, ObjSerialize, Object};

use super::{Project, ObjBox, action::ObjAction, frame::Frame, obj::{Obj, ObjPtr, child_obj::ChildObj, ObjList, ObjClone, ObjSerialize, ObjPtrAny}, graphic::Graphic};

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct Layer {
    #[parent]
    pub graphic: ObjPtr<Graphic>,
    #[field]
    pub name: String,
    #[field]
    pub show: bool,
    pub frames: Vec<ObjBox<Frame>>
}

impl Layer {

    pub fn get_frame_at(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        let mut best_frame = None;
        let mut best_time = -1;
        for frame in &self.frames {
            if frame.get(project).time <= time && frame.get(project).time > best_time {
                best_frame = Some(frame);
                best_time = frame.get(project).time;
            }
        }
        best_frame
    }

    pub fn get_frame_exactly_at(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        for frame in &self.frames {
            if frame.get(project).time == time {
                return Some(frame);
            }
        }
        None
    }

    pub fn get_frame_before(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        let mut best_frame = None;
        let mut best_time = -1;
        for frame in &self.frames {
            if frame.get(project).time < time && frame.get(project).time > best_time {
                best_frame = Some(frame);
                best_time = frame.get(project).time;
            }
        }
        best_frame
    }
    
    pub fn get_frame_after(&self, project: &Project, time: i32) -> Option<&ObjBox<Frame>> {
        let mut best_frame = None;
        let mut best_time = i32::MAX;
        for frame in &self.frames {
            if frame.get(project).time > time && frame.get(project).time < best_time {
                best_frame = Some(frame);
                best_time = frame.get(project).time;
            }
        }
        best_frame
    }

}

impl ChildObj for Layer {
    type Parent = Graphic;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.graphic
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.layers
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.layers
    }

}

impl Default for Layer {

    fn default() -> Self {
        Self {
            graphic: ObjPtr::null(),
            name: "Layer".to_owned(),
            show: true,
            frames: Vec::new()
        }
    }

}
