
use std::{cell::RefCell, collections::{HashMap, HashSet}, marker::PhantomData, sync::{Arc, Mutex}};
use super::{Obj, ObjBox, ObjPtr};

pub trait ObjListTrait {

    type ObjType: Obj;

    fn next_ptr(&mut self) -> ObjPtr<Self::ObjType>;
    fn curr_key(&mut self) -> &mut u64;

    fn add(&mut self, obj: Self::ObjType) -> ObjBox<Self::ObjType>; 
    fn add_with_ptr(&mut self, obj: Self::ObjType, ptr: ObjPtr<Self::ObjType>) -> ObjBox<Self::ObjType>;
    fn get(&self, ptr: ObjPtr<Self::ObjType>) -> Option<&Self::ObjType>;
    fn get_mut(&mut self, ptr: ObjPtr<Self::ObjType>) -> Option<&mut Self::ObjType>;

    fn use_obj_file_ptrs<F, R>(&self, f: F) -> R where F: Fn(&mut HashMap<ObjPtr<Self::ObjType>, u64>) -> R; 

    fn get_then<F, R>(&self, ptr: ObjPtr<Self::ObjType>, callback: F) -> Option<R> where F: FnOnce(&Self::ObjType) -> R {
        self.get(ptr).map(callback)
    }

    fn get_then_mut<F, R>(&mut self, ptr: ObjPtr<Self::ObjType>, callback: F) -> Option<R> where F: FnOnce(&mut Self::ObjType) -> R {
        self.get_mut(ptr).map(callback)
    }

    fn get_dropped(&self) -> Arc<Mutex<Vec<u64>>>;
    fn get_created(&self) -> &HashSet<ObjPtr<Self::ObjType>>;
    fn get_modified(&self) -> &HashSet<ObjPtr<Self::ObjType>>;

    fn mutated(&self) -> bool {
        !self.get_dropped().lock().unwrap().is_empty() || !self.get_modified().is_empty()
    }

}

pub struct ObjList<T: Obj> {
    pub objs: HashMap<u64, T>,

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


}

impl<T: Obj> ObjListTrait for ObjList<T> {

    type ObjType = T;

    fn next_ptr(&mut self) -> ObjPtr<T> {
        self.curr_key += 1;
        ObjPtr {
            key: self.curr_key - 1,
            _marker: PhantomData
        }
    }

    fn curr_key(&mut self) -> &mut u64 {
        &mut self.curr_key
    }

    fn add(&mut self, obj: T) -> ObjBox<T> {
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

    fn add_with_ptr(&mut self, obj: T, ptr: ObjPtr<T>) -> ObjBox<T> {
        self.objs.insert(ptr.key, obj);
        self.created.insert(ptr);
        self.modified.insert(ptr);
        ObjBox {
            ptr,
            dropped: self.dropped.clone(),
        }
    }

    fn get(&self, ptr: ObjPtr<T>) -> Option<&T> {
        self.objs.get(&ptr.key)
    }

    fn get_mut(&mut self, ptr: ObjPtr<T>) -> Option<&mut T> {
        self.modified.insert(ptr);
        self.objs.get_mut(&ptr.key)
    }

    fn use_obj_file_ptrs<F, R>(&self, f: F) -> R where F: Fn(&mut HashMap<ObjPtr<Self::ObjType>, u64>) -> R {
        let mut ptrs = self.obj_file_ptrs.borrow_mut();
        f(&mut ptrs)
    } 

    fn get_dropped(&self) -> Arc<Mutex<Vec<u64>>> {
        self.dropped.clone()
    }

    fn get_created(&self) -> &HashSet<ObjPtr<Self::ObjType>> {
        &self.created
    }

    fn get_modified(&self) -> &HashSet<ObjPtr<Self::ObjType>> {
        &self.modified
    }

}
