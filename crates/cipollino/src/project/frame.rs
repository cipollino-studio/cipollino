
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

    fn get_sibling_list(project: &mut Project, parent: ObjPtr<Self::Parent>) -> Option<&mut Vec<ObjBox<Frame>>> {
        if let Some(layer) = project.layers.get_mut(parent) {
            Some(&mut layer.frames)
        } else {
            None
        }
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
