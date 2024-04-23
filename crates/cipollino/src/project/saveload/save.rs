

use serde_json::json;

use crate::{project::{graphic::Graphic, obj::{child_obj::HasRootAsset, ObjBox}}, util::fs::write_json_file};

use super::asset_file::AssetFile;

use super::super::{folder::Folder, frame::Frame, layer::Layer, obj::{asset::Asset, Obj, ObjPtr, ObjSerialize}, palette::{Palette, PaletteColor}, sound_instance::SoundInstance, stroke::Stroke, Project};
use super::super::obj::obj_list::ObjListTrait;

impl Project {

    pub fn save<F>(&mut self, log_error: &mut F) where F: FnMut(String) {
        write_json_file(&self.save_path.clone().with_file_name("proj.cip"), json!({
            "fps": self.fps,
            "sample_rate": self.sample_rate,
            "audio_files": self.audio_files.save_lookups()
        }));

        self.create_asset_files(&self.root_folder, log_error);

        self.save_obj_list_modifications::<Stroke, F>(log_error);
        self.save_obj_list_modifications::<SoundInstance, F>(log_error);
        self.save_obj_list_modifications::<Frame, F>(log_error);
        self.save_obj_list_modifications::<Layer, F>(log_error);
        self.save_obj_list_modifications::<Graphic, F>(log_error);

        self.save_obj_list_modifications::<PaletteColor, F>(log_error);
        self.save_obj_list_modifications::<Palette, F>(log_error);
    }

    fn create_asset_file<T: Asset>(&self, obj_box: &ObjBox<T>) -> Result<(), String> {
        let path = obj_box.get_path(self).ok_or("Could not get path to asset.")?;
        if !path.exists() {
            let obj = obj_box.get(self); 
            let mut asset_file = AssetFile::create(path, obj_box.make_ptr().key, &T::type_magic_bytes())?;
            T::get_list(self).use_obj_file_ptrs(|ptrs| ptrs.insert(obj_box.make_ptr(), asset_file.root_obj_ptr));
            let data = obj.obj_serialize_full(self, &mut asset_file);
            asset_file.set_obj_data(asset_file.root_obj_ptr, data.as_document().expect("asset should serialize to bson document").clone())?;
        }
        Ok(())
    }

    fn create_asset_files<F>(&self, folder_box: &ObjBox<Folder>, log_error: &mut F) where F: FnMut(String) {
        let folder = folder_box.get(self); 
        let path = if let Some(path) = folder.file_path(self) {
            path
        } else {
            log_error("Could not get target folder path.".to_string());
            return;
        };
        if !path.exists() {
            if let Err(msg) = std::fs::create_dir(path).map_err(|err| err.to_string()) {
                log_error(format!("Could not create folder: {}", msg));
                return;
            }
        }

        for gfx in &folder.graphics {
            if let Err(msg) = self.create_asset_file(gfx) {
                log_error(msg);
            }
        }
        for palette in &folder.palettes {
            if let Err(msg) = self.create_asset_file(palette) {
                log_error(msg);
            }
        }
        for subfolder in &folder.folders {
            self.create_asset_files(subfolder, log_error);
        }

    }

    fn save_obj_creation<T: HasRootAsset + ObjSerialize>(&mut self, obj_ptr: ObjPtr<T>) -> Result<(), String> {
        if T::get_list(self).use_obj_file_ptrs(|ptrs| ptrs.get(&obj_ptr).is_some()) {
            return Ok(())
        } 
        let root_asset_ptr = T::get_root_asset(self, obj_ptr).ok_or("Asset missing.")?;
        let root_asset_path = T::RootAsset::get_list(self).get(root_asset_ptr).ok_or("Could not get asset.")?.file_path(self).ok_or("Could not get path to asset.")?;
        let mut asset_file = AssetFile::open(root_asset_path, &T::RootAsset::type_magic_bytes(), T::RootAsset::type_name())?;
        let page = asset_file.alloc_page()?;
        T::get_list_mut(self).use_obj_file_ptrs(|ptrs| ptrs.insert(obj_ptr, page));

        Ok(())
    }

    fn save_obj_modification<T: HasRootAsset + ObjSerialize>(&self, obj_ptr: ObjPtr<T>) -> Result<(), String> {
        let obj = if let Some(obj) = T::get_list(self).get(obj_ptr) {
            obj
        } else {
            return Ok(());
        };
        let root_asset_ptr = T::get_root_asset(self, obj_ptr).ok_or("Asset missing")?;
        let root_asset_path = T::RootAsset::get_list(self).get(root_asset_ptr).ok_or("Could not get asset")?.file_path(self).ok_or("Could not get path to asset.")?;
        let mut asset_file = AssetFile::open(root_asset_path, &T::RootAsset::type_magic_bytes(), T::RootAsset::type_name())?;

        let ptr = T::get_list(self).use_obj_file_ptrs(|ptrs| {
            if let Some(ptr) = ptrs.get(&obj_ptr) {
                Some(*ptr)
            } else {
                None
            }
        });
        let ptr = if let Some(ptr) = ptr {
            ptr
        } else {
            let ptr = asset_file.alloc_page()?;
            T::get_list(self).use_obj_file_ptrs(|ptrs| ptrs.insert(obj_ptr, ptr));
            ptr
        };

        let data = obj.obj_serialize(self, &mut asset_file).as_document().expect("objects should serialize to bson documents.").clone();
        let _ = asset_file.set_obj_data(ptr, data);
        Ok(())
    }

    fn save_obj_deletion<T: HasRootAsset + ObjSerialize>(&self, obj_ptr: ObjPtr<T>, delete_obj_ptrs: &mut Vec<ObjPtr<T>>) -> Result<(), String> {
        if let Some(ptr) = T::get_list(self).use_obj_file_ptrs(|ptrs| ptrs.get(&obj_ptr).map(|ptr| *ptr)) {
            let root_asset_ptr = T::get_root_asset(self, obj_ptr).ok_or("Asset missing.")?;
            let root_asset_path = T::RootAsset::get_list(self).get(root_asset_ptr).ok_or("Could not get asset.")?.file_path(self).ok_or("Could not get path to asset.")?;
            let mut asset_file = AssetFile::open(root_asset_path, &T::RootAsset::type_magic_bytes(), T::RootAsset::type_name())?;
            let _ = asset_file.delete_obj(ptr);
            delete_obj_ptrs.push(obj_ptr);
        }
        Ok(())
    }

    fn save_obj_list_modifications<T: HasRootAsset + ObjSerialize, F>(&mut self, log_error: &mut F) where F: FnMut(String) {

        let list = T::get_list(self);
        let mut delete_obj_ptrs = Vec::new();
        for key in &*list.get_dropped().lock().unwrap() {
            if let Err(msg) = self.save_obj_deletion(ObjPtr::<T>::from_key(*key), &mut delete_obj_ptrs) {
                log_error(msg);
            } 
        }

        let list = T::get_list_mut(self);
        for obj_ptr in delete_obj_ptrs {
            list.use_obj_file_ptrs(|ptrs| ptrs.remove(&obj_ptr));
        }

        let list = T::get_list(self);
        let created = list.get_created().clone();
        for obj in &created {
            if let Err(msg) = self.save_obj_creation(*obj) {
                log_error(msg);
            }
        }

        let list = T::get_list(self);
        for obj in list.get_modified() {
            if let Err(msg) = self.save_obj_modification(*obj) {
                log_error(msg);
            }
        }

    }

}