
use std::{collections::{HashMap, HashSet}, path::PathBuf, sync::Arc};

use bson::Bson;

use crate::project::{folder::Folder, saveload::{asset_file::AssetFile, load::LoadingMetadata}, Project};

use super::{asset::Asset, obj_list::{ObjList, ObjListTrait}, ObjBox, ObjPtr};

pub struct AssetList<T: Asset> {
    objs: ObjList<T>,
    pub to_load: HashMap<ObjPtr<T>, (ObjPtr<Folder>, String)>
}

impl<T: Asset> AssetList<T> {

    pub fn new() -> Self {
        Self {
            objs: ObjList::new(),
            to_load: HashMap::new()
        }
    }

    pub fn garbage_collect_objs(&mut self) {
        self.objs.garbage_collect_objs();
    }

    pub fn get_name(&self, ptr: ObjPtr<T>) -> Option<String> {
        if let Some(obj) = self.objs.get(ptr) {
            Some(obj.name().clone())
        } else if let Some((_, name)) = self.to_load.get(&ptr) {
            Some(name.clone())
        } else {
            None
        }
    }

    pub fn get_path(&self, ptr: ObjPtr<T>, project: &Project) -> Option<PathBuf> {
        if let Some(obj) = self.objs.get(ptr) {
            obj.file_path(project)
        } else if let Some((folder, name)) = self.to_load.get(&ptr) {
            let folder = project.folders.get(*folder)?;
            let folder_path = folder.file_path(project)?;
            Some(folder_path.join(format!("{}.{}", name, T::extension())))
        } else {
            None
        }
    }

    pub fn load(project: &mut Project, ptr: ObjPtr<T>, metadata: &mut LoadingMetadata) -> Result<(), String> {
        let list = T::get_list_mut(project);
        let (folder_ptr, name) = if let Some(path) = list.to_load.remove(&ptr) {
            path
        } else {
            return Ok(());
        };
        
        let folder = project.folders.get(folder_ptr).ok_or("Folder missing.")?; 
        let path = folder.file_path(project).ok_or("Could not get path.")?.join(format!("{}.{}", name, T::extension())); 
        let list = T::get_list_mut(project);
        let mut asset_file = AssetFile::open(path, &T::type_magic_bytes(), T::type_name())?; 
        list.objs.obj_file_ptrs.borrow_mut().insert(ptr, asset_file.root_obj_ptr);
        let obj_data = asset_file.get_obj_data(asset_file.root_obj_ptr)?; 

        let mut obj = T::obj_deserialize(project, &Bson::Document(obj_data), ptr.into(), &mut asset_file, metadata).ok_or("Could not deserialize.")?;
        *obj.parent_mut() = folder_ptr; 
        *obj.name_mut() = name;
        let list = T::get_list_mut(project);
        list.objs.objs.insert(ptr.key, obj);

        Ok(())
    }

}

impl<T: Asset> ObjListTrait for AssetList<T> {
    type ObjType = T;

    fn next_ptr(&mut self) -> super::ObjPtr<Self::ObjType> {
        self.objs.next_ptr()
    }

    fn curr_key(&mut self) -> &mut u64 {
        self.objs.curr_key()
    }

    fn add(&mut self, obj: Self::ObjType) -> super::ObjBox<Self::ObjType> {
        self.objs.add(obj)
    }

    fn add_with_ptr(&mut self, obj: Self::ObjType, ptr: ObjPtr<Self::ObjType>) -> ObjBox<Self::ObjType> {
        self.objs.add_with_ptr(obj, ptr)
    }

    fn get(&self, ptr: ObjPtr<Self::ObjType>) -> Option<&Self::ObjType> {
        self.objs.get(ptr)
    }

    fn get_mut(&mut self, ptr: ObjPtr<Self::ObjType>) -> Option<&mut Self::ObjType> {
        self.objs.get_mut(ptr)
    }

    fn use_obj_file_ptrs<F, R>(&self, f: F) -> R where F: Fn(&mut HashMap<ObjPtr<Self::ObjType>, u64>) -> R {
        self.objs.use_obj_file_ptrs(f)
    }

    fn get_dropped(&self) -> Arc<std::sync::Mutex<Vec<u64>>> {
        self.objs.get_dropped()
    }

    fn get_created(&self) -> &HashSet<super::ObjPtr<Self::ObjType>> {
        self.objs.get_created()
    }

    fn get_modified(&self) -> &HashSet<super::ObjPtr<Self::ObjType>> {
        self.objs.get_modified()
    }

}
