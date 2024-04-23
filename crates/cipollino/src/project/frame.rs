
use project_macros::{ObjClone, ObjSerialize, Object};
use unique_type_id::UniqueTypeId;

use super::{action::ObjAction, graphic::Graphic, layer::Layer, obj::{child_obj::{ChildObj, HasRootAsset}, Obj, ObjBox, ObjClone, ObjPtr, ObjSerialize}, stroke::Stroke, Project};
use crate::project::obj::obj_list::ObjListTrait;

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
pub struct Frame {
    #[parent]
    pub layer: ObjPtr<Layer>,
    #[field]
    pub time: i32,
    pub strokes: Vec<ObjBox<Stroke>>
}

impl ChildObj for Frame {
    type Parent = ObjPtr<Layer>;

    fn parent(&self) -> Self::Parent {
        self.layer
    }

    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.layer
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        Some(&project.layers.get(parent)?.frames)
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        Some(&mut project.layers.get_mut(parent)?.frames)
    }

}

impl HasRootAsset for Frame {

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
