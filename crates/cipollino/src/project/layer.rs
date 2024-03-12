
use project_macros::{ObjClone, ObjSerialize, Object};

use super::{action::ObjAction, frame::Frame, graphic::Graphic, obj::{child_obj::ChildObj, obj_clone_impls::PrimitiveObjClone, Obj, ObjClone, ObjList, ObjPtr, ObjPtrAny, ObjSerialize}, sound_instance::SoundInstance, ObjBox, Project};

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum LayerKind {
    Animation,
    Audio
}

impl PrimitiveObjClone for LayerKind {}

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct Layer {
    #[parent]
    pub graphic: ObjPtr<Graphic>,
    #[field]
    pub name: String,
    #[field]
    pub show: bool,
    #[field]
    pub kind: LayerKind,
    pub frames: Vec<ObjBox<Frame>>,
    pub sound_instances: Vec<ObjBox<SoundInstance>>
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
            kind: LayerKind::Animation,
            frames: Vec::new(),
            sound_instances: Vec::new()
        }
    }

}
