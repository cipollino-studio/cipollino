
pub mod folder;
pub mod graphic;
pub mod layer;
pub mod frame;
pub mod stroke;
pub mod obj;
pub mod action;
pub mod saveload;
use std::{collections::HashSet, path::PathBuf};

use self::{folder::Folder, frame::Frame, graphic::Graphic, layer::Layer, obj::{ObjBox, ObjList, ObjPtr}, stroke::Stroke};

pub struct Project {
    pub folders: ObjList<Folder>,
    pub graphics: ObjList<Graphic>,
    pub layers: ObjList<Layer>,
    pub frames: ObjList<Frame>,
    pub strokes: ObjList<Stroke>,

    pub root_folder: ObjBox<Folder>,

    pub save_path: Option<PathBuf>,
    pub files_to_delete: HashSet<PathBuf>
}

impl Project {

    pub fn new() -> Self {
        let mut folder_list = ObjList::new();
        let root = folder_list.add(Folder::new(ObjPtr::null()));
        Self {
            folders: folder_list,
            graphics: ObjList::new(),
            layers: ObjList::new(),
            frames: ObjList::new(),
            strokes: ObjList::new(),
            root_folder: root,
            save_path: None,
            files_to_delete: HashSet::new()
        }
    }

    pub fn garbage_collect_objs(&mut self) {
        self.folders.garbage_collect_objs();
        self.graphics.garbage_collect_objs();
        self.layers.garbage_collect_objs();
        self.frames.garbage_collect_objs();
        self.strokes.garbage_collect_objs();
    }

}
