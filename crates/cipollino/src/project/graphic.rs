
use project_macros::Object;

use super::{Project, action::ObjAction, obj::{Obj, ObjPtr, ObjBox, ObjList, ObjClone}, layer::Layer};

#[derive(Object, Clone)]
pub struct Graphic {
    #[field]
    pub name: String,
    #[field]
    pub len: u32,
    #[field]
    pub clip: bool,
    #[field]
    pub w: u32,
    #[field]
    pub h: u32,
    pub layers: Vec<ObjBox<Layer>>
}

impl Default for Graphic {

    fn default() -> Self {
        Self {
            name: "Graphic".to_owned(),
            len: 100,
            clip: false,
            w: 1920,
            h: 1080,
            layers: Vec::new()
        }
    }
}
