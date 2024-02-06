
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

use super::{action::ObjAction, Project};

mod obj_clone_impls;

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

pub trait ChildObj: Obj + 'static {
    type Parent: Obj;

    fn get_sibling_list(project: &mut Project, parent: ObjPtr<Self::Parent>) -> Option<&mut Vec<ObjBox<Self>>>;

    fn add_at_idx(project: &mut Project, parent: ObjPtr<Self::Parent>, obj: Self, idx: i32) -> Option<(ObjPtr<Self>, ObjAction)> {
        if let None = Self::get_sibling_list(project, parent) {
            return None;
        }
        let obj_box = Self::get_list_mut(project).add(obj);
        let obj_ptr = obj_box.make_ptr();
        let orig_obj_store = Rc::new(RefCell::new(Some(obj_box)));

        let obj_store = orig_obj_store.clone();
        let redo = move |proj: &'_ mut Project| {
            if let Some(siblings) = Self::get_sibling_list(proj, parent) {
                let obj = obj_store.replace(None).unwrap(); 
                let idx = if siblings.len() == 0 {
                    0
                } else if idx < 0 {
                    siblings.len() - ((-idx as usize) % siblings.len())
                } else {
                    (idx as usize) % siblings.len()
                };
                siblings.insert(idx, obj);
            }
        };

        let obj_store = orig_obj_store.clone();
        let undo = move |proj: &'_ mut Project| {
            if let Some(siblings) = Self::get_sibling_list(proj, parent) {
                let idx = siblings.iter().position(|other_obj| other_obj.make_ptr() == obj_ptr).unwrap();
                let obj = siblings.remove(idx);
                obj_store.replace(Some(obj));
            }
        };

        redo(project);

        return Some((obj_ptr, ObjAction::new(redo, undo)));
    }

    fn add(project: &mut Project, parent: ObjPtr<Self::Parent>, obj: Self) -> Option<(ObjPtr<Self>, ObjAction)> {
        Self::add_at_idx(project, parent, obj, -1)
    }

    fn delete(project: &mut Project, parent: ObjPtr<Self::Parent>, obj: ObjPtr<Self>) -> Option<ObjAction> {
        let siblings = Self::get_sibling_list(project, parent);
        if let None = siblings {
            return None;
        }
        let siblings = siblings.unwrap();
        if let Some(idx) = siblings.iter().position(|other_obj| other_obj.make_ptr() == obj) {
            let orig_obj_store = Rc::new(RefCell::new(None));

            let obj_store = orig_obj_store.clone();
            let redo = move |proj: &'_ mut Project| {
                if let Some(siblings) = Self::get_sibling_list(proj, parent) {
                    let obj_box = siblings.remove(idx);
                    obj_store.replace(Some(obj_box));
                }
            };

            let obj_store = orig_obj_store.clone();
            let undo = move |proj: &'_ mut Project| {
                if let Some(siblings) = Self::get_sibling_list(proj, parent) {
                    let obj_box = obj_store.replace(None).unwrap();
                    siblings.insert(idx, obj_box);
                }
            };

            redo(project);

            return Some(ObjAction::new(redo, undo));
        }
        None
    }

    fn get_box(project: &mut Project, parent: ObjPtr<Self::Parent>, obj: ObjPtr<Self>) -> Option<&ObjBox<Self>> {
        let siblings = Self::get_sibling_list(project, parent)?;
        for sibling in siblings {
            if sibling.make_ptr() == obj {
                return Some(sibling);
            }
        }
        None
    }

}
