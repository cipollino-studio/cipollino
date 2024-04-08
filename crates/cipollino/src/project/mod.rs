
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

use std::{collections::HashSet, path::PathBuf};

use serde_json::json;

use crate::util::fs::write_json_file;

use self::{file::{audio::AudioFile, FileList, FilePtr}, folder::Folder, frame::Frame, graphic::Graphic, layer::Layer, obj::{ObjBox, ObjList, ObjPtr}, palette::{Palette, PaletteColor}, sound_instance::SoundInstance, stroke::Stroke};

pub struct Project {
    pub fps: f32,
    pub sample_rate: f32,

    pub folders: ObjList<Folder>,
    pub graphics: ObjList<Graphic>,
    pub layers: ObjList<Layer>,
    pub frames: ObjList<Frame>,
    pub strokes: ObjList<Stroke>,
    pub palettes: ObjList<Palette>,
    pub palette_colors: ObjList<PaletteColor>,
    pub sound_instances: ObjList<SoundInstance>,

    pub audio_files: FileList<AudioFile>, 

    pub root_folder: ObjBox<Folder>,

    // Path to the proj.cip file at the root of the project folder
    pub save_path: PathBuf,

    // List of strokes whose mesh needs to be updated 
    pub remeshes_needed: HashSet<ObjPtr<Stroke>>
}

impl Project {

    pub fn create(path: PathBuf, fps: f32, sample_rate: f32) -> Self {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        write_json_file(&path, json!({
            "fps": fps,
            "sample_rate": sample_rate
        }));
        Self::load(path).0
    }

    pub fn new(path: PathBuf, fps: f32, sample_rate: f32) -> Self {
        let mut folder_list = ObjList::new();
        let root = folder_list.add(Folder::new(ObjPtr::null()));

        Self {
            fps,
            sample_rate,
            folders: folder_list,
            graphics: ObjList::new(),
            layers: ObjList::new(),
            frames: ObjList::new(),
            strokes: ObjList::new(),
            palettes: ObjList::new(),
            palette_colors: ObjList::new(),
            sound_instances: ObjList::new(),

            audio_files: FileList::new(),

            root_folder: root,

            save_path: path,

            remeshes_needed: HashSet::new()
        }
    }

    pub fn mutated(&self) -> bool {
        self.folders.mutated() || self.graphics.mutated() || self.layers.mutated() || self.frames.mutated() || self.strokes.mutated() || self.palettes.mutated() || self.palette_colors.mutated() || self.sound_instances.mutated()
    }

    pub fn garbage_collect_objs(&mut self) {
        self.folders.garbage_collect_objs();
        self.graphics.garbage_collect_objs();
        self.layers.garbage_collect_objs();
        self.frames.garbage_collect_objs();
        self.strokes.garbage_collect_objs();
        self.sound_instances.garbage_collect_objs();
        self.palettes.garbage_collect_objs();
        self.palette_colors.garbage_collect_objs();
    }

}

#[derive(PartialEq, Eq, Clone)]
pub enum AssetPtr {
    Folder(ObjPtr<Folder>),
    Graphic(ObjPtr<Graphic>),
    Palette(ObjPtr<Palette>),
    Audio(FilePtr<AudioFile>)
}
