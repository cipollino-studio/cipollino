
use project_macros::{ObjClone, ObjSerialize, Object};

use super::{action::ObjAction, graphic::Graphic, layer::Layer, obj::{child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjList, ObjPtr, ObjPtrAny, ObjSerialize}, stroke::Stroke, Project};

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct Frame {
    #[parent]
    pub layer: ObjPtr<Layer>,
    #[field]
    pub time: i32,
    pub strokes: Vec<ObjBox<Stroke>>
}

impl ChildObj for Frame {
    type Parent = Layer;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.layer
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.frames
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.frames
    }

    type RootAsset = Graphic;
    fn get_root_asset(project: &Project, frame: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        Layer::get_root_asset(project, project.frames.get(frame)?.layer)
    }

}

impl Default for Frame {

    fn default() -> Self {
        Self {
            layer: ObjPtr::null(),
            time: 0,
            strokes: Vec::new()
        }
    }

}

impl Frame {

    pub fn frame_set_time(project: &mut Project, frame_ptr: ObjPtr<Frame>, time: i32) -> Option<Vec<ObjAction>> {
        let frame = project.frames.get(frame_ptr)?;
        let layer = project.layers.get(frame.layer)?;
        let mut acts = Vec::new();
        for other_frame in &layer.frames {
            if other_frame.make_ptr() != frame_ptr && other_frame.get(&project).time == time {
                acts.push(Frame::delete(project, other_frame.make_ptr())?);
                break;
            }
        }
        acts.push(Frame::set_time(project, frame_ptr, time)?);
        Some(acts)
    }

}
