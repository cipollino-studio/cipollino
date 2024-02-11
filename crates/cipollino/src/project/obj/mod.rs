
use std::{collections::HashMap, marker::PhantomData};
use super::Project;

mod obj_clone_impls;
pub mod asset;
pub mod child_obj;

pub struct ObjList<T: Obj> {
    objs: HashMap<u64, T>, 
    pub curr_key: u64
}

impl<T: Obj> ObjList<T> {

    pub fn new() -> Self {
        Self {
            objs: HashMap::new(),
            curr_key: 1
        }
    }

    pub fn add(&mut self, obj: T) -> ObjBox<T> {
        self.objs.insert(self.curr_key, obj);
        self.curr_key += 1;
        ObjBox {
            ptr: ObjPtr {
                key: self.curr_key - 1,
                _marker: PhantomData
            } 
        }
    }

    pub fn add_with_ptr(&mut self, obj: T, ptr: ObjPtr<T>) -> ObjBox<T> {
        self.objs.insert(ptr.key, obj);
        ObjBox {
            ptr
        }
    }

    pub fn get(&self, ptr: ObjPtr<T>) -> Option<&T> {
        self.objs.get(&ptr.key)
    }

    pub fn get_mut(&mut self, ptr: ObjPtr<T>) -> Option<&mut T> {
        self.objs.get_mut(&ptr.key)
    }

    pub fn get_then<F, R>(&self, ptr: ObjPtr<T>, callback: F) -> Option<R> where F: FnOnce(&T) -> R {
        self.get(ptr).map(callback)
    }

    pub fn get_then_mut<F, R>(&mut self, ptr: ObjPtr<T>, callback: F) -> Option<R> where F: FnOnce(&mut T) -> R {
        self.get_mut(ptr).map(callback)
    }

}

pub struct ObjPtr<T: Obj> {
    pub key: u64,
    _marker: PhantomData<T>
}

impl<T: Obj> ObjPtr<T> {

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

#[derive(Clone)]
pub struct ObjBox<T: Obj> {
    ptr: ObjPtr<T>
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

pub trait ObjClone : Clone {

    fn obj_clone(&self, _project: &mut Project) -> Self {
        self.clone()
    }

    fn obj_serialize(&self, project: &Project) -> serde_json::Value;

    fn obj_deserialize(project: &mut Project, data: &serde_json::Value) -> Option<Self>;

}

pub trait Obj: Sized + ObjClone {

    fn get_list(project: &Project) -> &ObjList<Self>;
    fn get_list_mut(project: &mut Project) -> &mut ObjList<Self>;

}
