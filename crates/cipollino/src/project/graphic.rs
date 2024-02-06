
use project_macros::{ObjClone, Object};

use super::{action::ObjAction, layer::Layer, obj::{Obj, ObjPtr, ObjBox, ObjList, ObjClone}, saveload::Asset, Project};

#[derive(Object, Clone, ObjClone)]
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

impl Asset for Graphic {

    fn name(&self) -> String {
        self.name.clone()
    }

}