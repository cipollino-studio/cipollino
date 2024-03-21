
pub mod folder;
pub mod graphic;
pub mod layer;
pub mod frame;
pub mod stroke;
pub mod obj;
pub mod action;
pub mod saveload;
pub mod palette;
pub mod sound_instance;
pub mod file;

use std::{collections::{HashMap, HashSet}, path::PathBuf};

use self::{file::{audio::AudioFile, FilePtr, FilePtrAny}, folder::Folder, frame::Frame, graphic::Graphic, layer::Layer, obj::{ObjBox, ObjList, ObjPtr}, palette::{Palette, PaletteColor}, sound_instance::SoundInstance, stroke::Stroke};

pub struct Project {
    pub folders: ObjList<Folder>,
    pub graphics: ObjList<Graphic>,
    pub layers: ObjList<Layer>,
    pub frames: ObjList<Frame>,
    pub strokes: ObjList<Stroke>,
    pub palettes: ObjList<Palette>,
    pub palette_colors: ObjList<PaletteColor>,
    pub sound_instances: ObjList<SoundInstance>,

    pub audio_files: HashMap<FilePtr<AudioFile>, AudioFile>,

    pub path_file_ptr: HashMap<PathBuf, FilePtrAny>,
    pub hash_file_ptr: HashMap<u64, FilePtrAny>,

    pub root_folder: ObjBox<Folder>,

    pub save_path: Option<PathBuf>,
    pub files_to_delete: HashSet<PathBuf>,
    pub files_to_move: Vec<(PathBuf, PathBuf)> 
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
            palettes: ObjList::new(),
            palette_colors: ObjList::new(),
            sound_instances: ObjList::new(),
            audio_files: HashMap::new(),
            path_file_ptr: HashMap::new(),
            hash_file_ptr: HashMap::new(),
            root_folder: root,
            save_path: None,
            files_to_delete: HashSet::new(),
            files_to_move: Vec::new()
        }
    }

    pub fn garbage_collect_objs(&mut self) {
        self.folders.garbage_collect_objs();
        self.graphics.garbage_collect_objs();
        self.layers.garbage_collect_objs();
        self.frames.garbage_collect_objs();
        self.strokes.garbage_collect_objs();
        self.palettes.garbage_collect_objs();
        self.palette_colors.garbage_collect_objs();
        self.sound_instances.garbage_collect_objs();
    }

}

#[derive(PartialEq, Eq, Clone)]
pub enum TypedAssetPtr {
    Folder(ObjPtr<Folder>),
    Graphic(ObjPtr<Graphic>),
    Palette(ObjPtr<Palette>),
    Audio(FilePtr<AudioFile>)
}
