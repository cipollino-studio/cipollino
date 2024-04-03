

use serde_json::json;

use crate::{project::{graphic::Graphic, obj::ObjBox}, util::fs::write_json_file};

use super::asset_file::AssetFile;

use super::super::{folder::Folder, frame::Frame, layer::Layer, obj::{asset::Asset, child_obj::ChildObj, Obj, ObjPtr, ObjSerialize}, palette::{Palette, PaletteColor}, sound_instance::SoundInstance, stroke::Stroke, Project};

impl Project {

    pub fn save(&mut self) {
        write_json_file(&self.save_path.clone().with_file_name("proj.cip"), json!({
            "fps": self.fps,
            "sample_rate": self.sample_rate,
            "audio_files": self.audio_files.save_lookups()
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

}