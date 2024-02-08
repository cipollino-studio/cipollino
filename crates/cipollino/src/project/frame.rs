
use project_macros::{ObjClone, Object};

use super::{Project, action::ObjAction, stroke::Stroke, obj::Obj, obj::{ObjBox, ObjList, ObjClone}, obj::ChildObj, obj::ObjPtr, layer::Layer};

#[derive(Object, Clone, ObjClone)]
pub struct Frame {
    pub layer: ObjPtr<Layer>,
    #[field]
    pub time: i32,
    pub strokes: Vec<ObjBox<Stroke>>
}

impl ChildObj for Frame {
    type Parent = Layer;

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.frames
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.frames
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
