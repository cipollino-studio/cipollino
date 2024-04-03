
use std::{cell::RefCell, collections::{HashMap, HashSet}, marker::PhantomData, sync::{Arc, Mutex}};
use std::hash::Hash;

use super::{saveload::{asset_file::AssetFile, load::LoadingMetadata}, Project};

pub mod obj_clone_impls;
pub mod asset;
pub mod child_obj;


pub struct ObjList<T: Obj> {
    objs: HashMap<u64, T>,

    /*
        When an ObjBox is dropped, we want to automatically destroy the object it contained.
        A reference to list is given to each ObjBox, allowing the list to "garbage collect" deleted objects.
    */
    pub dropped: Arc<Mutex<Vec<u64>>>,

    pub created: HashSet<ObjPtr<T>>,
    pub modified: HashSet<ObjPtr<T>>,

    pub curr_key: u64,
    
    // First page pointers of every object in their respective asset files(see saveload::asset_file)
    pub obj_file_ptrs: RefCell<HashMap<ObjPtr<T>, u64>>
}

impl<T: Obj> ObjList<T> {

    pub fn new() -> Self {
        Self {
            objs: HashMap::new(),
            dropped: Arc::new(Mutex::new(Vec::new())),
            created: HashSet::new(),
            modified: HashSet::new(),
            curr_key: 1,
            obj_file_ptrs: RefCell::new(HashMap::new())
        }
    }

    pub fn next_ptr(&mut self) -> ObjPtr<T> {
        self.curr_key += 1;
        ObjPtr {
            key: self.curr_key - 1,
            _marker: PhantomData
        }
    }

    pub fn add(&mut self, obj: T) -> ObjBox<T> {
        self.objs.insert(self.curr_key, obj);
        self.curr_key += 1;
        let ptr = ObjPtr {
            key: self.curr_key - 1,
            _marker: PhantomData
        };
        self.created.insert(ptr);
        self.modified.insert(ptr);
        ObjBox {
            ptr,
            dropped: self.dropped.clone(),
        }
    }

    pub fn add_with_ptr(&mut self, obj: T, ptr: ObjPtr<T>) -> ObjBox<T> {
        self.objs.insert(ptr.key, obj);
        self.created.insert(ptr);
        self.modified.insert(ptr);
        ObjBox {
            ptr,
            dropped: self.dropped.clone(),
        }
    }

    pub fn get(&self, ptr: ObjPtr<T>) -> Option<&T> {
        self.objs.get(&ptr.key)
    }

    pub fn get_mut(&mut self, ptr: ObjPtr<T>) -> Option<&mut T> {
        self.modified.insert(ptr);
        self.objs.get_mut(&ptr.key)
    }

    pub fn get_then<F, R>(&self, ptr: ObjPtr<T>, callback: F) -> Option<R> where F: FnOnce(&T) -> R {
        self.get(ptr).map(callback)
    }

    pub fn get_then_mut<F, R>(&mut self, ptr: ObjPtr<T>, callback: F) -> Option<R> where F: FnOnce(&mut T) -> R {
        self.get_mut(ptr).map(callback)
    }

    pub fn garbage_collect_objs(&mut self) {
        loop {
            let mut dropped = self.dropped.lock().unwrap();
            let dropped_clone = dropped.clone();
            dropped.clear();
            drop(dropped);

            for key in dropped_clone {
                self.objs.remove(&key);
            }

            if self.dropped.lock().unwrap().is_empty() {
                break;
            }
        }

        self.created.clear();
        self.modified.clear();
    } 

    pub fn mutated(&self) -> bool {
        !self.dropped.lock().unwrap().is_empty() || !self.modified.is_empty()
    }

}

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
pub struct ObjPtrAny(u64);

impl<T: Obj> From<ObjPtrAny> for ObjPtr<T> {

    fn from(value: ObjPtrAny) -> Self {
        ObjPtr {
            key: value.0,
            _marker: PhantomData
        }
    }

}

impl<T: Obj> From<ObjPtr<T>> for ObjPtrAny {

    fn from(value: ObjPtr<T>) -> Self {
        Self(value.key)
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
    ptr: ObjPtr<T>,
    dropped: Arc<Mutex<Vec<u64>>>,
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
    fn obj_deserialize(project: &mut Project, data: &bson::Bson, parent: ObjPtrAny, asset_file: &mut AssetFile, metadata: &mut LoadingMetadata) -> Option<Self>;

    // Used to take data out of the object tree, for undo/redo
    type RawData: Send + Sync;
    fn to_raw_data(&self, project: &Project) -> Self::RawData;
    fn from_raw_data(project: &mut Project, data: &Self::RawData) -> Self;

}

pub trait Obj: Sized + ObjClone + Send + Sync {

    fn get_list(project: &Project) -> &ObjList<Self>;
    fn get_list_mut(project: &mut Project) -> &mut ObjList<Self>;

}
