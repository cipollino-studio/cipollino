
use std::path::PathBuf;

use project_macros::{ObjClone, Object};
use crate::project::obj::{Obj, ObjList};
use crate::project::Project;
use super::graphic::Graphic;
use super::obj::{ObjBox, ObjClone};


#[derive(Object, Clone, ObjClone)]
pub struct Folder {
    pub graphics: Vec<ObjBox<Graphic>>
}

impl Folder {

    pub fn new() -> Self {
        Self {
            graphics: Vec::new()
        }
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::new()
    }

}

impl Default for Folder {
    fn default() -> Self {
        Self::new()
    }
}
