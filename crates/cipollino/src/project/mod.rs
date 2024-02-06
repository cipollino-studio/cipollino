
pub mod graphic;
pub mod layer;
pub mod frame;
pub mod stroke;
pub mod obj;
pub mod action;
pub mod saveload;
use std::path::PathBuf;

use self::{frame::Frame, graphic::Graphic, layer::Layer, obj::{ObjBox, ObjList}, stroke::Stroke};

pub struct Project {
    pub graphics: ObjList<Graphic>,
    pub layers: ObjList<Layer>,
    pub frames: ObjList<Frame>,
    pub strokes: ObjList<Stroke>,

    pub root_graphics: Vec<ObjBox<Graphic>>,

    pub save_path: Option<PathBuf>
}

impl Project {

    pub fn new() -> Self {
        Self {
            graphics: ObjList::new(),
            layers: ObjList::new(),
            frames: ObjList::new(),
            strokes: ObjList::new(),
            root_graphics: Vec::new(),
            save_path: None
        }
    }

}
