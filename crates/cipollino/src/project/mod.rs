
pub mod graphic;
pub mod layer;
pub mod frame;
pub mod stroke;
pub mod obj;
pub mod action;

use self::{frame::Frame, graphic::Graphic, layer::Layer, obj::{ObjBox, ObjList}, stroke::Stroke};


pub struct Project {
    pub graphics: ObjList<Graphic>,
    pub layers: ObjList<Layer>,
    pub frames: ObjList<Frame>,
    pub strokes: ObjList<Stroke>,

    pub root_graphics: Vec<ObjBox<Graphic>>
}

impl Project {

    pub fn new() -> Self {
        let mut res = Self {
            graphics: ObjList::new(),
            layers: ObjList::new(),
            frames: ObjList::new(),
            strokes: ObjList::new(),
            root_graphics: Vec::new()
        };

        res.root_graphics.push(res.graphics.add(Graphic {
            name: "Clip".to_owned(),
            len: 100,
            clip: true,
            w: 1920,
            h: 1080,
            layers: vec![res.layers.add(Layer {
                name: "Layer".to_owned(),
                frames: Vec::new()
            })]
        }));

        res
    }

}
