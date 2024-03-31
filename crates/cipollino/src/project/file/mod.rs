
use std::{hash::Hash, marker::PhantomData, path::PathBuf};

use crate::util::{bson::{bson_get, bson_to_u64, u64_to_bson}, next_unique_name};

use super::{action::ObjAction, folder::Folder, obj::{obj_clone_impls::PrimitiveObjClone, ObjClone, ObjPtr, ObjSerialize}, saveload::asset_file::AssetFile, AssetPtr, Project};

pub mod audio;

// TODO: This system has no chance of functioning. Rewrite.

#[derive(Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FilePtrAny {
    pub path: PathBuf,
    pub hash: u64,
}

impl FilePtrAny {

    pub fn new(path: PathBuf, hash: u64) -> Self {
        Self {
            path,
            hash
        }
    }

    pub fn null() -> Self {
        Self {
            path: PathBuf::new(),
            hash: 0,
        }
    }

    pub fn name(&self) -> &str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }

    pub fn set_name(&mut self, name: String) {
        self.path.set_file_name(format!("{}.{}", name, self.path.extension().unwrap().to_str().unwrap()));
    }

    pub fn lookup(&self, project: &Project) -> Self {
        if !project.path_file_ptr.contains_key(&self.path) {
            if let Some(new_ptr) = project.hash_file_ptr.get(&self.hash) {
                return new_ptr.clone();
            }
        }
        self.clone()
    }

}

pub trait FileType: Sized + Send + Sync + Clone {

    fn list_in_folder(folder: &Folder) -> &Vec<FilePtr<Self>>;
    fn list_in_folder_mut(folder: &mut Folder) -> &mut Vec<FilePtr<Self>>;

    fn make_asset_ptr(ptr: &FilePtr<Self>) -> AssetPtr;

    fn icon() -> &'static str;
    
    fn rename(project: &mut Project, mut ptr: FilePtr<Self>, folder_ptr: ObjPtr<Folder>, name: String) -> Option<ObjAction> where Self: 'static {
        let old_ptr = ptr.clone();
        let from_path = project.base_path().join(&ptr.ptr.path);

        let folder = project.folders.get(folder_ptr)?;
        let idx = Self::list_in_folder(folder).iter().position(|other| *other == ptr)?;
        let new_name = next_unique_name(&name, folder.audios.iter().map(|audio| audio.name()));
        let to_path = from_path.with_file_name(format!("{}.{}", new_name, from_path.extension().unwrap().to_str().unwrap()));

        std::fs::rename(from_path.clone(), to_path.clone()).ok()?;

        ptr.ptr.path = to_path.clone();
        let new_ptr = ptr.clone();
        let folder = project.folders.get_mut(folder_ptr)?;
        let list = Self::list_in_folder_mut(folder);
        list.remove(idx);
        list.push(ptr.clone());

        let redo = {
            let from_path = from_path.clone();
            let to_path = to_path.clone();
            let old_ptr = old_ptr.clone();
            let new_ptr = new_ptr.clone();
            move |proj: &mut Project| {
                let _ = std::fs::rename(from_path.clone(), to_path.clone());
                if let Some(folder) = proj.folders.get_mut(folder_ptr) {
                    let list = Self::list_in_folder_mut(folder); 
                    if let Some(idx) = list.iter().position(|other| other == &old_ptr) {
                        list.remove(idx);
                    } 
                    list.push(new_ptr.clone());
                }
            }
        };

        let undo = {
            move |proj: &mut Project| {
                let _ = std::fs::rename(to_path.clone(), from_path.clone());
                if let Some(folder) = proj.folders.get_mut(folder_ptr) {
                    let list = Self::list_in_folder_mut(folder); 
                    if let Some(idx) = list.iter().position(|other| other == &new_ptr) {
                        list.remove(idx);
                    } 
                    list.push(old_ptr.clone());
                }
            }
        };

        Some(ObjAction::new(redo, undo))
    }

    fn transfer(project: &mut Project, mut ptr: FilePtr<Self>, from: ObjPtr<Folder>, to: ObjPtr<Folder>) -> Option<ObjAction> where Self: 'static {
        let old_ptr = ptr.clone();
        let from_folder = project.folders.get(from)?;
        let mut from_path = from_folder.file_path(project)?;
        from_path.push(ptr.ptr.path.file_name()?);
        let idx = Self::list_in_folder(from_folder).iter().position(|other| *other == ptr)?;

        let to_folder = project.folders.get(to)?;
        let mut to_path = to_folder.file_path(project)?;
        let new_name = next_unique_name(&ptr.name().to_owned(), Self::list_in_folder(to_folder).iter().map(|ptr| ptr.name()));
        ptr.set_name(new_name);
        let new_ptr = ptr.clone();
        to_path.push(ptr.ptr.path.file_name()?);

        let to_folder = project.folders.get_mut(to)?;
        Self::list_in_folder_mut(to_folder).push(ptr.clone());
        
        let from_folder = project.folders.get_mut(from)?;
        Self::list_in_folder_mut(from_folder).remove(idx);

        let _ = std::fs::rename(from_path.clone(), to_path.clone());

        let redo = {
            let from_path = from_path.clone();
            let to_path = to_path.clone();
            let old_ptr = old_ptr.clone();
            let new_ptr = new_ptr.clone();
            move |proj: &mut Project| {
                let _ = std::fs::rename(from_path.clone(), to_path.clone());
                if let Some(from_folder) = proj.folders.get_mut(from) {
                    let list = Self::list_in_folder_mut(from_folder);
                    if let Some(idx) = list.iter().position(|other| other == &old_ptr) {
                        list.remove(idx);
                    }
                }
                if let Some(to_folder) = proj.folders.get_mut(to) {
                    let list = Self::list_in_folder_mut(to_folder);
                    list.push(new_ptr.clone()); 
                }
            }
        };

        let undo = {
            let from_path = from_path.clone();
            let to_path = to_path.clone();
            let old_ptr = old_ptr.clone();
            let new_ptr = new_ptr.clone();
            move |proj: &mut Project| {
                let _ = std::fs::rename(to_path.clone(), from_path.clone());
                if let Some(to_folder) = proj.folders.get_mut(to) {
                    let list = Self::list_in_folder_mut(to_folder);
                    if let Some(idx) = list.iter().position(|other| other == &new_ptr) {
                        list.remove(idx);
                    }
                }
                if let Some(from_folder) = proj.folders.get_mut(from) {
                    let list = Self::list_in_folder_mut(from_folder);
                    list.push(old_ptr.clone()); 
                }
            }
        };

        Some(ObjAction::new(redo, undo)) 
    }

}

#[derive(Clone, serde::Deserialize)]
pub struct FilePtr<T: FileType> {
    pub ptr: FilePtrAny,
    pub _marker: PhantomData<T>
}

impl<T: FileType> PartialEq for FilePtr<T> {

    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }

}

impl<T: FileType> Eq for FilePtr<T> {}

impl<T: FileType> Hash for FilePtr<T> {

    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }

}

impl<T: FileType> FilePtr<T> {

    pub fn new(path: PathBuf, hash: u64) -> Self {
        Self {
            ptr: FilePtrAny::new(path, hash),
            _marker: PhantomData
        }
    }

    pub fn null() -> Self {
        Self {
            ptr: FilePtrAny::null(),
            _marker: PhantomData
        }
    }

    pub fn name(&self) -> &str {
        self.ptr.name()
    }

    pub fn set_name(&mut self, name: String) {
        self.ptr.set_name(name);
    }

    pub fn lookup(&self, project: &Project) -> Self {
        Self {
            ptr: self.ptr.lookup(project),
            _marker: PhantomData
        }
    }

}

impl<T: FileType + Clone> ObjClone for FilePtr<T> {} 
impl PrimitiveObjClone for PathBuf {}
impl PrimitiveObjClone for [u8; 8] {}

impl<T: FileType> ObjSerialize for FilePtr<T> {

    fn obj_serialize(&self, project: &Project, _asset_file: &mut AssetFile) -> bson::Bson {
        let ptr = self.lookup(project);
        bson::bson! ({
            "filepath": ptr.ptr.path.to_str().unwrap(),
            "hash": u64_to_bson(ptr.ptr.hash)
        })
    }

    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson {
        self.obj_serialize(project, asset_file)
    }

    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: super::obj::ObjPtrAny, asset_file: &mut AssetFile) -> Option<Self> {
        let path = PathBuf::obj_deserialize(project, bson_get(&data, "filepath")?, parent, asset_file)?;
        let hash = bson_to_u64(bson_get(data, "hash")?)?;
        Some(Self {
            ptr: FilePtrAny::new(path, hash),
            _marker: PhantomData
        })
    }

    type RawData = Self;
    fn to_raw_data(&self, _project: &Project) -> Self::RawData {
        self.clone() 
    }

    fn from_raw_data(_project: &mut Project, data: &Self::RawData) -> Self {
        data.clone() 
    }

}
