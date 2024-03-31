
use std::{fs, path::PathBuf};

use bson::bson;
use serde_json::json;

use crate::{project::{graphic::Graphic, obj::ObjBox}, util::{bson::u64_to_bson, fs::{read_json_file, write_json_file}}};
use sha2::Digest;

use self::asset_file::AssetFile;

use super::{file::{audio::AudioFile, FilePtr, FileType}, folder::Folder, frame::Frame, layer::Layer, obj::{asset::Asset, child_obj::ChildObj, Obj, ObjPtr, ObjSerialize}, palette::{Palette, PaletteColor}, sound_instance::SoundInstance, stroke::Stroke, Project};

pub mod asset_file;

impl Project {

    pub fn save(&mut self) {
        write_json_file(&self.save_path.clone().with_file_name("proj.cip"), json!({
            "fps": self.fps,
            "sample_rate": self.sample_rate
        }));

        self.create_asset_files(&self.root_folder);

        self.save_obj_list_modifications::<Stroke>();
        self.save_obj_list_modifications::<SoundInstance>();
        self.save_obj_list_modifications::<Frame>();
        self.save_obj_list_modifications::<Layer>();
        self.save_obj_list_modifications::<Graphic>();

        self.save_obj_list_modifications::<PaletteColor>();
        self.save_obj_list_modifications::<Palette>();
    }

    fn create_asset_file<T: Asset>(&self, obj_box: &ObjBox<T>) -> Option<()> {
        let obj = obj_box.get(self); 
        let path = obj.file_path(self)?;
        if !path.exists() {
            let mut asset_file = AssetFile::create(path, obj_box.make_ptr().key).ok()?;
            T::get_list(self).obj_file_ptrs.borrow_mut().insert(obj_box.make_ptr(), asset_file.root_obj_ptr);
            let data = obj.obj_serialize_full(self, &mut asset_file);
            asset_file.set_obj_data(asset_file.root_obj_ptr, data.as_document().expect("asset should serialize to bson document").clone()).ok()?;
        }
        Some(())
    }

    fn create_asset_files(&self, folder_box: &ObjBox<Folder>) -> Option<()>{
        let folder = folder_box.get(self); 
        let path = folder.file_path(self)?;
        if !path.exists() {
            std::fs::create_dir(path).ok()?;
        }

        for gfx in &folder.graphics {
            self.create_asset_file(gfx);
        }
        for palette in &folder.palettes {
            self.create_asset_file(palette);
        }
        for subfolder in &folder.folders {
            self.create_asset_files(subfolder);
        }

        Some(())
    }

    fn save_obj_creation<T: ChildObj + ObjSerialize>(&mut self, obj_ptr: ObjPtr<T>) -> Option<()> {
        if T::get_list(self).obj_file_ptrs.borrow().get(&obj_ptr).is_some() {
            return Some(())
        } 
        let root_asset_ptr = T::get_root_asset(self, obj_ptr)?;
        let root_asset_path = T::RootAsset::get_list(self).get(root_asset_ptr)?.file_path(self)?;
        let mut asset_file = AssetFile::open(root_asset_path).ok()?;
        T::get_list_mut(self).obj_file_ptrs.borrow_mut().insert(obj_ptr, asset_file.alloc_page().ok()?);

        Some(())
    }

    fn save_obj_modification<T: ChildObj + ObjSerialize>(&self, obj_ptr: ObjPtr<T>) -> Option<()> {
        let obj = T::get_list(self).get(obj_ptr)?;
        let root_asset_ptr = T::get_root_asset(self, obj_ptr)?;
        let root_asset_path = T::RootAsset::get_list(self).get(root_asset_ptr)?.file_path(self)?;
        let mut asset_file = AssetFile::open(root_asset_path).ok()?;

        let ptr = *T::get_list(self).obj_file_ptrs.borrow().get(&obj_ptr)?;
        let data = obj.obj_serialize(self, &mut asset_file).as_document().expect("objects should serialize to bson documents.").clone();
        let _ = asset_file.set_obj_data(ptr, data);
        Some(())
    }

    fn save_obj_deletion<T: ChildObj + ObjSerialize>(&self, obj_ptr: ObjPtr<T>, delete_obj_ptrs: &mut Vec<ObjPtr<T>>) -> Option<()> {
        if let Some(ptr) = T::get_list(self).obj_file_ptrs.borrow().get(&obj_ptr) {
            let root_asset_ptr = T::get_root_asset(self, obj_ptr)?;
            let root_asset_path = T::RootAsset::get_list(self).get(root_asset_ptr)?.file_path(self)?;
            let mut asset_file = AssetFile::open(root_asset_path).ok()?;
            let _ = asset_file.delete_obj(*ptr);
            delete_obj_ptrs.push(obj_ptr);
        }
        Some(())
    }

    fn save_obj_list_modifications<T: ChildObj + ObjSerialize>(&mut self) -> Option<()> {

        let list = T::get_list(self);
        let mut delete_obj_ptrs = Vec::new();
        for key in &*list.dropped.lock().unwrap() {
            self.save_obj_deletion(ObjPtr::<T>::from_key(*key), &mut delete_obj_ptrs); 
        }

        let list = T::get_list_mut(self);
        for obj_ptr in delete_obj_ptrs {
            list.obj_file_ptrs.borrow_mut().remove(&obj_ptr);
        }

        let list = T::get_list(self);
        let created = list.created.clone();
        for obj in &created {
            self.save_obj_creation(*obj);
        }

        let list = T::get_list(self);
        for obj in &list.modified {
            self.save_obj_modification(*obj); 
        }

        Some(())
    }

    pub fn load(proj_file_path: PathBuf) -> Self {
        let mut fps = 24.0;
        let mut sample_rate = 44100.0;
        if let Some(proj_data) = read_json_file(&proj_file_path) {
            if let Some(new_fps) = proj_data.get("fps").map_or(None, |val| val.as_f64()) {
                fps = new_fps as f32;
            }
            if let Some(new_sample_rate) = proj_data.get("sample_rate").map_or(None, |val| val.as_f64()) {
                sample_rate = new_sample_rate as f32;
            }
        }

        let mut res = Self::new(proj_file_path.clone(), fps, sample_rate);

        let mut base_folder_path = proj_file_path.clone();
        base_folder_path.pop();

        let folder_path = proj_file_path.parent().unwrap();
        res.root_folder = res.load_folder(&folder_path.to_owned(), ObjPtr::null()); 

        res
    }

    fn load_folder(&mut self, path: &PathBuf, parent: ObjPtr<Folder>) -> ObjBox<Folder> {
        let res = self.folders.add(Folder::new(parent));
        res.get_mut(self).name = path.file_name().unwrap().to_str().unwrap().to_owned();

        if let Ok(paths) = fs::read_dir(path) {
            for path in paths {
                if let Ok(path) = path {
                    let path = path.path();
                    self.load_file(path, res.make_ptr());
                }
            }
        }
        res
    } 

    fn load_asset<T: Asset + ObjSerialize>(&mut self, path: PathBuf, folder: ObjPtr<Folder>) -> Option<()> {
        let mut asset_file = AssetFile::open(path.clone()).ok()?;
        T::get_list_mut(self).obj_file_ptrs.borrow_mut().insert(ObjPtr::from_key(asset_file.root_obj_key), asset_file.root_obj_ptr);
        let obj_box = ObjBox::<T>::obj_deserialize(self, &bson!({
            "key": u64_to_bson(asset_file.root_obj_key),
            "ptr": u64_to_bson(asset_file.root_obj_ptr) 
        }), folder.into(), &mut asset_file)?;

        *obj_box.get_mut(self).name_mut() = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let folder = self.folders.get_mut(folder).unwrap();
        T::get_list_in_parent_mut(folder).push(obj_box);

        Some(())
    }

    fn load_file(&mut self, path: PathBuf, folder_ptr: ObjPtr<Folder>) { 
        if self.folders.get_mut(folder_ptr).is_none() {
            return;
        }

        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap() {
                "cipgfx" => {
                    let _ = self.load_asset::<Graphic>(path.clone(), folder_ptr);
                },
                "cippal" => {
                    let _ = self.load_asset::<Palette>(path.clone(), folder_ptr);
                },
                "mp3" => {
                    let file_ptr = self.get_file_ptr(&path, &self.base_path());
                    let data = AudioFile::load(&path);
                    if let Some(file_ptr) = file_ptr {
                        if let Some(data) = data {
                            let folder = self.folders.get_mut(folder_ptr).unwrap();
                            folder.audios.push(file_ptr.clone());
                            self.audio_files.insert(file_ptr, data);
                        }
                    }
                },
                _ => {}
            }
        } 
        if path.is_dir() {
            let sub_folder = self.load_folder(&path, folder_ptr);
            let folder = self.folders.get_mut(folder_ptr).unwrap();
            folder.folders.push(sub_folder);
        }
    }

    pub fn load_file_to_root_folder(&mut self, path: PathBuf) {
        self.load_file(path, self.root_folder.make_ptr());
    }

    pub fn get_file_ptr<T: FileType>(&mut self, path: &PathBuf, base: &PathBuf) -> Option<FilePtr<T>> {
        let rel_path = pathdiff::diff_paths(path, base)?; 
        let mut hash = sha2::Sha256::new();
        hash.update(fs::read(path).ok()?);
        let hash = hash.finalize();
        let mut hash_val: u64 = 0;
        for i in 0..8 {
            hash_val <<= 8;
            hash_val |= hash[i] as u64;
        }

        let file_ptr = FilePtr::<T>::new(rel_path.clone(), hash_val); 
    
        self.path_file_ptr.insert(rel_path, file_ptr.ptr.clone());
        self.hash_file_ptr.insert(hash_val, file_ptr.ptr.clone());

        Some(file_ptr)
    }

    pub fn base_path(&self) -> PathBuf {
        self.save_path.parent().unwrap().to_owned()
    }

}
