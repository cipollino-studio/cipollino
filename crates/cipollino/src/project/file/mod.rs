
use std::{hash::Hash, marker::PhantomData, path::PathBuf};

use serde_json::json;

use crate::util::next_unique_name;

use super::{action::ObjAction, folder::Folder, obj::{obj_clone_impls::PrimitiveObjClone, ObjClone, ObjPtr, ObjSerialize}, Project};

pub mod audio;

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

    fn transfer(project: &mut Project, mut ptr: FilePtr<Self>, from: ObjPtr<Folder>, to: ObjPtr<Folder>) -> Option<ObjAction> where Self: 'static {
        // TODO: Fix undo when file is forcefully renamed

        let _old_name = ptr.name().to_owned();

        let from_folder = project.folders.get(from)?;
        let mut from_path = from_folder.file_path(project)?;
        from_path.push(ptr.ptr.path.file_name()?);
        let idx = Self::list_in_folder(from_folder).iter().position(|other| *other == ptr)?;

        let to_folder = project.folders.get(to)?;
        let mut to_path = to_folder.file_path(project)?;
        let new_name = next_unique_name(&ptr.name().to_owned(), Self::list_in_folder(to_folder).iter().map(|ptr| ptr.name()));
        ptr.set_name(new_name);
        to_path.push(ptr.ptr.path.file_name()?);

        let to_folder = project.folders.get_mut(to)?;
        Self::list_in_folder_mut(to_folder).push(ptr.clone());
        
        let from_folder = project.folders.get_mut(from)?;
        Self::list_in_folder_mut(from_folder).remove(idx);

        project.files_to_move.push((from_path, to_path));

        let ptr_1 = ptr.clone();
        Some(ObjAction::new(move |proj| {
            Self::transfer(proj, ptr_1.clone(), from, to);
        }, move |proj| {
            Self::transfer(proj, ptr.clone(), to, from);
        })) 
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

    fn obj_serialize(&self, project: &Project) -> serde_json::Value {
        let ptr = self.lookup(project);
        json!({
            "filepath": ptr.ptr.path,
            "hash": ptr.ptr.hash
        })
    }

    fn obj_deserialize(project: &mut Project, data: &serde_json::Value, parent: super::obj::ObjPtrAny) -> Option<Self> {
        let path = PathBuf::obj_deserialize(project, data.get("filepath")?, parent)?;
        let hash = data.get("hash")?.as_u64()?; 
        Some(Self {
            ptr: FilePtrAny::new(path, hash),
            _marker: PhantomData
        })
    }

}
