
use std::{cell::RefCell, collections::{HashMap, HashSet}, marker::PhantomData, path::PathBuf, sync::{Arc, Mutex}};

use serde_json::{json, Map, Value};

use std::hash::Hash;

use crate::util::{bson::{bson_get, bson_to_u64, u64_to_bson}, fs::{set_file_stem, trash_folder}, next_unique_name};

use super::{action::ObjAction, folder::Folder, obj::{obj_list::ObjListTrait, ObjClone, ObjPtr, ObjSerialize, ToRawData}, saveload::{asset_file::AssetFile, load::LoadingMetadata}, AssetPtr, Project};

pub mod audio;

pub trait ResourceType: Sized + Send + Sync + Clone {

    fn load(project: &Project, path: PathBuf) -> Result<Self, String>;

    fn get_list(project: &Project) -> &ResourceList<Self>;
    fn get_list_mut(project: &mut Project) -> &mut ResourceList<Self>;
    fn list_in_folder(folder: &Folder) -> &Vec<ResPtr<Self>>;
    fn list_in_folder_mut(folder: &mut Folder) -> &mut Vec<ResPtr<Self>>;
    fn list_in_loading_metadata(metadata: &LoadingMetadata) -> &HashSet<ResPtr<Self>>;
    fn list_in_loading_metadata_mut(metadata: &mut LoadingMetadata) -> &mut HashSet<ResPtr<Self>>; 

    fn make_asset_ptr(ptr: &ResPtr<Self>) -> AssetPtr;

    fn icon() -> &'static str;

    fn delete(project: &mut Project, ptr: ResPtr<Self>) -> Option<ObjAction> where Self: 'static {
        let resource = ptr.get(project)?;
        let path = resource.path.clone();
        let absolute_path = resource.absolute_path(project);
        let key = ptr.key;
        Self::list_in_folder_mut(project.folders.get_mut(resource.folder)?).retain(|other_ptr| *other_ptr != ptr);
        Self::get_list_mut(project).path_lookup.remove(&path)?;
        let resource = Self::get_list_mut(project).resources.remove(&ptr.key)?;

        let mut graveyard_path = trash_folder().join(absolute_path.file_name()?);
        if graveyard_path.exists() {
            for i in 1.. {
                graveyard_path = trash_folder().join(absolute_path.file_name()?);
                set_file_stem(&mut graveyard_path, format!("{} ({})", absolute_path.file_stem()?.to_str()?, i).as_str());
                if !graveyard_path.exists() {
                    break;
                }
            }
        }

        std::fs::rename(absolute_path.clone(), graveyard_path.clone()).ok()?;

        let file_box = Arc::new(Mutex::new(Some(resource)));
        let absolute_path_1 = absolute_path.clone();
        let graveyard_path_1 = graveyard_path.clone();
        let file_box_1 = file_box.clone();
        Some(ObjAction::new(move |proj| {
            if std::fs::rename(absolute_path.clone(), graveyard_path.clone()).is_ok() {
                let file = Self::get_list_mut(proj).resources.remove(&key).unwrap();
                Self::get_list_mut(proj).path_lookup.remove(&file.path);
                Self::list_in_folder_mut(proj.folders.get_mut(file.folder).unwrap()).retain(|other_file| other_file != &file.ptr);
                *file_box.lock().unwrap() = Some(file);
            }
        }, move |proj| {
            if std::fs::rename(graveyard_path_1.clone(), absolute_path_1.clone()).is_ok() {
                let file = std::mem::replace(&mut *file_box_1.lock().unwrap(), None).unwrap();
                Self::get_list_mut(proj).path_lookup.insert(path.clone(), key);
                Self::list_in_folder_mut(proj.folders.get_mut(file.folder).unwrap()).push(file.ptr);
                Self::get_list_mut(proj).resources.insert(file.ptr.key, file);
            }
        }))
    }

    fn rename(project: &mut Project, ptr: &ResPtr<Self>, name: String) -> Option<ObjAction> where Self: 'static {
        let file = Self::get_list(project).get(ptr)?;
        let new_name = next_unique_name(&name, Self::list_in_folder(project.folders.get(file.folder)?).iter().map(|file_ptr| file_ptr.get(project).map_or("", |file| file.name())));
        let old_name = file.name().to_owned(); 

        let old_path = file.absolute_path(project);
        let old_path_rel = file.path.clone();
        Self::get_list_mut(project).path_lookup.remove(&old_path_rel);
        let file = Self::get_list_mut(project).get_mut(ptr)?;
        set_file_stem(&mut file.path, &new_name);
        let file = Self::get_list(project).get(ptr)?;
        let key = file.ptr.key;
        let new_path_rel = file.path.clone();
        let new_path = file.absolute_path(project);
        Self::get_list_mut(project).path_lookup.insert(new_path_rel.clone(), key);

        std::fs::rename(old_path.clone(), new_path.clone()).ok()?;

        let ptr = ptr.clone();
        let ptr_1 = ptr.clone();
        let old_path_1 = old_path.clone();
        let new_path_1 = new_path.clone();
        let old_path_rel_1 = old_path_rel.clone();
        let new_path_rel_1 = new_path_rel.clone();
        Some(ObjAction::new(move |proj| {
            Self::get_list_mut(proj).path_lookup.remove(&old_path_rel);
            let file = Self::get_list_mut(proj).get_mut(&ptr).unwrap();
            set_file_stem(&mut file.path, &new_name);
            Self::get_list_mut(proj).path_lookup.insert(new_path_rel.clone(), key);
            let _ = std::fs::rename(old_path.clone(), new_path.clone());
        }, move |proj| {
            Self::get_list_mut(proj).path_lookup.remove(&new_path_rel_1);
            let file = Self::get_list_mut(proj).get_mut(&ptr_1).unwrap();
            set_file_stem(&mut file.path, &old_name);
            Self::get_list_mut(proj).path_lookup.insert(old_path_rel_1.clone(), key);
            let _ = std::fs::rename(new_path_1.clone(), old_path_1.clone());
        }))
    }

    fn transfer(project: &mut Project, ptr: &ResPtr<Self>, from: ObjPtr<Folder>, to: ObjPtr<Folder>) -> Option<ObjAction> where Self: 'static {
        if from == to {
            return None;
        }

        let file = Self::get_list(project).get(ptr)?;
        let new_name = next_unique_name(&file.name().to_owned(), Self::list_in_folder(project.folders.get(to)?).iter().map(|file_ptr| file_ptr.get(&project).map_or("", |file| file.name())));

        let to_folder = project.folders.get(to)?;
        let mut new_path = to_folder.file_path(project)?.join(file.path.file_name()?.to_str()?);
        set_file_stem(&mut new_path, &new_name);
        let new_path_rel = pathdiff::diff_paths(new_path.clone(), project.base_path())?;

        let old_path = file.absolute_path(project);
        let old_path_rel = file.path.clone();
        Self::get_list_mut(project).path_lookup.remove(&old_path_rel);

        let file = Self::get_list_mut(project).get_mut(ptr)?;
        file.path = new_path_rel.clone(); 

        let file = Self::get_list(project).get(ptr)?;
        let key = file.ptr.key;
        Self::get_list_mut(project).path_lookup.insert(new_path_rel.clone(), key);

        Self::list_in_folder_mut(project.folders.get_mut(from)?).retain(|other| other != ptr);
        Self::list_in_folder_mut(project.folders.get_mut(to)?).push(ptr.clone());
        std::fs::rename(old_path.clone(), new_path.clone()).ok()?;

        let ptr = ptr.clone();
        let ptr_1 = ptr.clone();
        let old_path_1 = old_path.clone();
        let new_path_1 = new_path.clone();
        let old_path_rel_1 = old_path_rel.clone();
        let new_path_rel_1 = new_path_rel.clone();
        Some(ObjAction::new(move |proj| {
            Self::get_list_mut(proj).path_lookup.remove(&old_path_rel);
            let file = Self::get_list_mut(proj).get_mut(&ptr).unwrap();
            file.path = new_path_rel.clone();
            Self::get_list_mut(proj).path_lookup.insert(new_path_rel.clone(), key);
            Self::list_in_folder_mut(proj.folders.get_mut(from).unwrap()).retain(|other| other != &ptr);
            Self::list_in_folder_mut(proj.folders.get_mut(to).unwrap()).push(ptr.clone());
            let _ = std::fs::rename(old_path.clone(), new_path.clone());
        }, move |proj| {
            Self::get_list_mut(proj).path_lookup.remove(&new_path_rel_1);
            let file = Self::get_list_mut(proj).get_mut(&ptr_1).unwrap();
            file.path = old_path_rel_1.clone();
            Self::get_list_mut(proj).path_lookup.insert(old_path_rel_1.clone(), key);
            Self::list_in_folder_mut(proj.folders.get_mut(to).unwrap()).retain(|other| other != &ptr_1);
            Self::list_in_folder_mut(proj.folders.get_mut(from).unwrap()).push(ptr_1.clone());
            let _ = std::fs::rename(new_path_1.clone(), old_path_1.clone());
        }))
    }

}

pub struct ResPtr<T: ResourceType> {
    key: u64, 
    _marker: PhantomData<T>
}

impl<T: ResourceType> ResPtr<T> {

    pub fn from_key(key: u64) -> Self {
        Self {
            key,
            _marker: PhantomData
        }
    }

    pub fn null() -> Self {
        Self::from_key(0)
    }

    pub fn get<'a>(&'a self, project: &'a Project) -> Option<&'a ResBox<T>> {
        T::get_list(project).get(self)
    }

    pub fn get_mut<'a>(&'a self, project: &'a mut Project) -> Option<&'a mut ResBox<T>> {
        T::get_list_mut(project).get_mut(self)
    }

}

impl<T: ResourceType> Hash for ResPtr<T> {

    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }

}

impl<T: ResourceType> Clone for ResPtr<T> {

    fn clone(&self) -> Self {
        Self {
            key: self.key,
            _marker: PhantomData
        }
    }

}

impl<T: ResourceType> Copy for ResPtr<T> {} 

impl<T: ResourceType> ObjClone for ResPtr<T> {}

impl<T: ResourceType> PartialEq for ResPtr<T> {

    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }

}

impl<T: ResourceType> Eq for ResPtr<T> {}

pub struct ResBox<T: ResourceType> {
    pub data: RefCell<Option<Arc<T>>>,
    pub ptr: ResPtr<T>,
    // Path to the file, relative to the root folder of the project
    pub path: PathBuf,
    pub folder: ObjPtr<Folder>
}

impl<T: ResourceType> ResBox<T> {
    
    pub fn name<'a>(&'a self) -> &'a str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }

    pub fn absolute_path(&self, project: &Project) -> PathBuf {
        project.base_path().join(self.path.clone())
    }

    pub fn get_data(&self, project: &Project) -> Option<Arc<T>> {
        let mut data = self.data.borrow_mut(); 
        if let Some(data) = &*data {
            return Some(data.clone());
        }

        let loaded_data = Arc::new(T::load(project, self.absolute_path(project)).ok()?); 
        *data = Some(loaded_data.clone());

        Some(loaded_data) 
    }

}

pub struct ResourceList<T: ResourceType> {
    resources: HashMap<u64, ResBox<T>>,
    pub path_lookup: HashMap<PathBuf, u64>,
    curr_key: u64
}

impl<T: ResourceType> ResourceList<T> {

    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            path_lookup: HashMap::new(),
            curr_key: 1
        }
    }

    pub fn load_lookups(&mut self, data: Value) -> Option<()> {

        let paths = data.get("paths")?.as_object()?;
        for (path, key) in paths.iter() { 
            let key = key.as_u64()?;
            self.path_lookup.insert(path.into(), key); 
            self.curr_key = self.curr_key.max(key + 1);
        }

        Some(())
    }

    pub fn save_lookups(&self) -> Value {
        let mut paths = Map::new();

        for (path, key) in self.path_lookup.iter() {
            paths.insert(path.to_str().unwrap().to_owned(), json!(*key));
        }

        json!({
            "paths": paths,
        })
    }

    pub fn load_resource(project: &mut Project, project_base_path: PathBuf, path: PathBuf, folder: ObjPtr<Folder>) -> Result<ResPtr<T>, String> {
        let rel_path = if let Some(rel_path) = pathdiff::diff_paths(path.clone(), project_base_path) {
            rel_path
        } else {
            return Err(format!("Invalid path {}.", path.to_string_lossy()));
        };
        let list = T::get_list_mut(project);

        let mut key = if let Some(key) = list.path_lookup.get(&rel_path) {
            *key
        } else {
            list.curr_key
        };
        list.curr_key = list.curr_key.max(key + 1);

        if list.resources.contains_key(&key) {
            key = list.curr_key;
            list.curr_key += 1; 
        }

        let ptr = ResPtr {
            key,
            _marker: PhantomData
        };

        list.resources.insert(key, ResBox {
            data: RefCell::new(None),
            ptr: ptr.clone(),
            path: rel_path.clone(),
            folder
        });
        list.path_lookup.insert(rel_path, key);

        Ok(ptr)
    }

    pub fn get<'a>(&'a self, ptr: &ResPtr<T>) -> Option<&'a ResBox<T>> {
        self.resources.get(&ptr.key)
    }
    
    pub fn get_mut<'a>(&'a mut self, ptr: &ResPtr<T>) -> Option<&'a mut ResBox<T>> {
        self.resources.get_mut(&ptr.key)
    }

}

impl<T: ResourceType> ObjSerialize for ResPtr<T> {

    fn obj_serialize(&self, _project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        bson::bson!({
            "key": u64_to_bson(self.key),
        })
    }

    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        self.obj_serialize(project, asset_file)
    }

    fn obj_deserialize(_project: &mut Project, data: &bson::Bson, parent: super::obj::DynObjPtr, _asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self> {
        let key = if let Some(key) = bson_get(data, "key") {
            key
        } else {
            metadata.deserialization_error("Resource key missing.", parent.key);
            return None;
        };
        let key = if let Some(key) = bson_to_u64(key) {
            key
        } else {
            metadata.deserialization_error("Resource key is not u64.", parent.key);
            return None;
        };
        let res = Self {
            key, 
            _marker: PhantomData
        };
        T::list_in_loading_metadata_mut(metadata).insert(res);
        Some(res)
    }


}

impl<T: ResourceType> ToRawData for ResPtr<T> {

    type RawData = Self;

    fn to_raw_data(&self, _project: &Project) -> Self::RawData {
        self.clone()
    }

    fn from_raw_data(_project: &mut Project, data: &Self::RawData) -> Self {
        data.clone() 
    }

}
