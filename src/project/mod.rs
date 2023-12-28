
pub mod graphic;
pub mod layer;
pub mod frame;
pub mod stroke;
pub mod point;

pub mod action;

use std::collections::HashMap;
use self::{graphic::{Graphic, GraphicData}, layer::Layer, frame::Frame, stroke::Stroke, point::Point};

pub trait ObjData {

    fn add(&self, key: u64, project: &mut Project);
    fn delete(&self, key: u64, project: &mut Project);
    fn set(&self, key: u64, project: &mut Project);

}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {

    pub graphics: HashMap<u64, Graphic>,
    pub layers: HashMap<u64, Layer>,
    pub frames: HashMap<u64, Frame>,
    pub strokes: HashMap<u64, Stroke>,
    pub points: HashMap<u64, Point>,

    curr_key: u64

}

impl Project {

    pub fn new() -> Self {
        let mut res = Self {
            graphics: HashMap::new(),
            layers: HashMap::new(),
            frames: HashMap::new(),
            strokes: HashMap::new(),
            points: HashMap::new(),
            curr_key: 1
        };
        let (gfx, _act) = res.add_graphic(GraphicData {
            name: "Clip".to_owned(),
            len: 100,
            clip: true,
            w: 1920,
            h: 1080
        });
        res.add_layer(gfx, "Layer".to_owned());
        res
    }

    pub fn next_key(&mut self) -> u64 {
        self.curr_key += 1;
        self.curr_key - 1
    }

}
