
use std::{marker::PhantomData, path::PathBuf, sync::{Arc, Mutex}};
use std::hash::Hash;

use unique_type_id::{TypeId, UniqueTypeId};

use self::{asset::Asset, asset_list::AssetList, obj_list::ObjListTrait};

use super::{saveload::{asset_file::AssetFile, load::LoadingMetadata}, Project};

pub mod obj_list;
pub mod child_obj;
pub mod obj_clone_impls;
pub mod asset;
pub mod asset_list;


#[derive(serde::Serialize, serde::Deserialize)]
pub struct ObjPtr<T: Obj> {
    pub key: u64,
    _marker: PhantomData<T>
}

impl<T: Obj> ObjPtr<T> {

    pub fn from_key(key: u64) -> Self {
        Self {
            key,
            _marker: PhantomData
        }
    }

    pub fn null() -> Self {
        Self {
            key: 0,
            _marker: PhantomData
        } 
    }

    pub fn make_obj_clone(&self, project: &mut Project) -> Option<T> {
        let obj = T::get_list(project).get(*self); 
        if obj.is_none() {
            return None;
        }
        let obj = obj.unwrap();
        Some(obj.clone().obj_clone(project))
    }

}

impl<T: Obj> std::fmt::Debug for ObjPtr<T> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ObjPtr").field(&self.key).finish()
    }

}

#[derive(Clone, Copy, Debug)]
pub struct DynObjPtr {
    pub key: u64,
    obj_type: TypeId<u64>
}

impl DynObjPtr {

    pub fn is<T: Obj>(&self) -> bool {
        self.obj_type == T::id()
    }

}

impl<T: Obj> From<DynObjPtr> for ObjPtr<T> {

    fn from(value: DynObjPtr) -> Self {
        assert!(value.obj_type == T::id(), "invalid obj ptr cast.");
        ObjPtr {
            key: value.key,
            _marker: PhantomData
        }
    }

}

impl<T: Obj> From<ObjPtr<T>> for DynObjPtr {

    fn from(value: ObjPtr<T>) -> Self {
        Self {
            key: value.key,
            obj_type: T::id()
        }
    }

}

impl<T: Obj> Clone for ObjPtr<T> {

    fn clone(&self) -> Self {
        Self {
            key: self.key,
            _marker: PhantomData
        }
    }

}

impl<T: Obj> Copy for ObjPtr<T> {}

impl<T: Obj> PartialEq for ObjPtr<T> {

    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }

}

impl<T: Obj> Eq for ObjPtr<T> {}

impl<T: Obj> Default for ObjPtr<T> {

    fn default() -> Self {
        Self::null() 
    }
    
}

impl<T: Obj> Hash for ObjPtr<T> {

    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }

}

#[derive(Clone)]
pub struct ObjBox<T: Obj> {
    pub ptr: ObjPtr<T>,
    pub dropped: Arc<Mutex<Vec<u64>>>,
}

impl<T: Obj> ObjBox<T> {

    pub fn get<'a>(&self, project: &'a Project) -> &'a T {
        T::get_list(project).get(self.ptr).unwrap()
    }

    pub fn get_mut<'a>(&self, project: &'a mut Project) -> &'a mut T {
        T::get_list_mut(project).get_mut(self.ptr).unwrap()
    }

    pub fn make_ptr(&self) -> ObjPtr<T> {
        self.ptr
    }

}

impl<T: Asset + Obj<ListType = AssetList<T>>> ObjBox<T> {

    pub fn get_name(&self, project: &Project) -> String {
        T::get_list(project).get_name(self.ptr).unwrap()
    }

    pub fn get_path(&self, project: &Project) -> Option<PathBuf> {
        T::get_list(project).get_path(self.ptr, project)
    }

}

impl<T: Obj> Drop for ObjBox<T> {

    fn drop(&mut self) {
        self.dropped.lock().unwrap().push(self.ptr.key);
    }

}

pub trait ObjClone : Clone {

    fn obj_clone(&self, _project: &mut Project) -> Self {
        self.clone()
    }
    
}

pub trait ObjSerialize : Sized {

    // Used to serialize one object at a time, for saving modifications incrementally
    fn obj_serialize(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson;
    // Used to write the entire object tree to disk, when creating an asset file
    fn obj_serialize_full(&self, project: &Project, asset_file: &mut AssetFile) -> bson::Bson;
    // Used to deserialize the entire object tree
    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: DynObjPtr, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self>;

}

// Used to take data out of the object tree, for undo/redo
pub trait ToRawData {
    
    type RawData: Send + Sync;
    fn to_raw_data(&self, project: &Project) -> Self::RawData;
    fn from_raw_data(project: &mut Project, data: &Self::RawData) -> Self;

}

pub trait Obj: Sized + ObjClone + Send + Sync + UniqueTypeId<u64> {

    type ListType: ObjListTrait<ObjType = Self>;

    fn get_list(project: &Project) -> &Self::ListType;
    fn get_list_mut(project: &mut Project) -> &mut Self::ListType;
    fn type_name() -> &'static str;

}
