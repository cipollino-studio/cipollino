
pub mod graphic;
pub mod layer;
pub mod action;
pub mod frame;

use std::collections::HashMap;
use self::{graphic::Graphic, layer::Layer, frame::Frame};

pub trait ObjData {

    fn add(&self, key: u64, project: &mut Project);
    fn delete(&self, key: u64, project: &mut Project);
    fn set(&self, key: u64, project: &mut Project);

}

pub struct Project {

    pub graphics: HashMap<u64, Graphic>,
    pub layers: HashMap<u64, Layer>,
    pub frames: HashMap<u64, Frame>,

    curr_key: u64

}

impl Project {

    pub fn new() -> Self {
        Self {
            graphics: HashMap::new(),
            layers: HashMap::new(),
            frames: HashMap::new(),
            curr_key: 1
        }
    }

    pub fn next_key(&mut self) -> u64 {
        self.curr_key += 1;
        self.curr_key - 1
    }

}
