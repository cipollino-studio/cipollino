use std::sync::{Arc, Mutex};

use crate::project::{action::ObjAction, Project};

use super::{asset::Asset, asset_list::AssetList, DynObjPtr, Obj, ObjBox, ObjPtr, ObjSerialize, ToRawData};

use crate::project::obj::obj_list::ObjListTrait;

pub trait ChildObj: Obj + 'static + ObjSerialize + ToRawData {
    type Parent: Copy + Send + Sync + Eq + From<DynObjPtr>;

    fn parent(&self) -> Self::Parent;
    fn parent_mut(&mut self) -> &mut Self::Parent;
    fn get_list_in_parent<'a>(project: &'a Project, parent: Self::Parent) -> Option<&'a Vec<ObjBox<Self>>>;
    fn get_list_in_parent_mut<'a>(project: &'a mut Project, parent: Self::Parent) -> Option<&'a mut Vec<ObjBox<Self>>>;

    fn add_at_idx(project: &mut Project, parent: Self::Parent, mut obj: Self, idx: i32) -> Option<(ObjPtr<Self>, ObjAction)> {
        if let None = Self::get_list_in_parent_mut(project, parent) {
            return None;
        }
        *obj.parent_mut() = parent;
        let obj_box = Self::get_list_mut(project).add(obj);
        let obj_ptr = obj_box.make_ptr();
        let orig_obj_store = Arc::new(Mutex::new(None));

        let siblings = Self::get_list_in_parent_mut(project, parent).unwrap();
        let idx = if siblings.len() == 0 {
            0
        } else if idx < 0 {
            siblings.len() - ((-(idx + 1) as usize) % siblings.len())
        } else {
            (idx as usize) % siblings.len()
        };
        siblings.insert(idx, obj_box);

        let obj_store = orig_obj_store.clone();
        let redo = move |proj: &'_ mut Project| {
            let data = std::mem::replace(&mut *obj_store.lock().unwrap(), None).unwrap();
            let obj_box = ObjBox::<Self>::from_raw_data(proj, &data);
            let siblings = Self::get_list_in_parent_mut(proj, parent).unwrap();
            siblings.insert(idx, obj_box);
        };

        let obj_store = orig_obj_store.clone();
        let undo = move |proj: &'_ mut Project| {
            let siblings = Self::get_list_in_parent_mut(proj, parent).unwrap();
            let obj_box = siblings.remove(idx);
            let _ = std::mem::replace(&mut *obj_store.lock().unwrap(), Some(obj_box.to_raw_data(proj)));
        };

        return Some((obj_ptr, ObjAction::new(redo, undo)));
    }

    fn add(project: &mut Project, parent: Self::Parent, obj: Self) -> Option<(ObjPtr<Self>, ObjAction)> {
        Self::add_at_idx(project, parent, obj, -1)
    }

    fn delete(project: &mut Project, obj: ObjPtr<Self>) -> Option<ObjAction> {
        let parent = *Self::get_list_mut(project).get_mut(obj)?.parent_mut();
        let siblings = Self::get_list_in_parent(project, parent)?;
        let idx = siblings.iter().position(|other_obj| other_obj.make_ptr() == obj)?;

        let orig_obj_store = Arc::new(Mutex::new(None));

        let obj_store = orig_obj_store.clone();
        let redo = move |proj: &'_ mut Project| {
            if let Some(siblings) = Self::get_list_in_parent_mut(proj, parent) {
                let obj_box = siblings.remove(idx);
                *obj_store.lock().unwrap() = Some(obj_box.to_raw_data(proj));
            }
        };

        let obj_store = orig_obj_store.clone();
        let undo = move |proj: &'_ mut Project| {
            let obj_box_data = std::mem::replace(&mut *obj_store.lock().unwrap(), None).unwrap(); 
            let obj_box = ObjBox::<Self>::from_raw_data(proj, &obj_box_data);
            let siblings = Self::get_list_in_parent_mut(proj, parent).unwrap();
            siblings.insert(idx, obj_box);
        };

        redo(project);

        Some(ObjAction::new(redo, undo))
    }

    fn get_box(project: &mut Project, parent: Self::Parent, obj: ObjPtr<Self>) -> Option<&ObjBox<Self>> {
        let siblings = Self::get_list_in_parent_mut(project, parent)?;
        for sibling in siblings {
            if sibling.make_ptr() == obj {
                return Some(sibling);
            }
        }
        None
    }

    fn transfer(project: &mut Project, obj_ptr: ObjPtr<Self>, new_parent: Self::Parent) -> Option<ObjAction> {
        let obj = Self::get_list_mut(project).get_mut(obj_ptr)?;
        if *obj.parent_mut() == new_parent {
            return None;
        }
        let init_parent = *obj.parent_mut();
        let idx = Self::get_list_in_parent(&project, init_parent)?.iter().position(|other_obj| other_obj.make_ptr() == obj_ptr)?;
        Self::get_list_in_parent(&project, new_parent)?;

        let redo = move |proj: &'_ mut Project| {
            let obj_box = Self::get_list_in_parent_mut(proj, init_parent).unwrap().remove(idx);
            Self::get_list_in_parent_mut(proj, new_parent).unwrap().push(obj_box);
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            *obj.parent_mut() = new_parent;
        };

        let undo = move |proj: &'_ mut Project| {
            let obj_box = Self::get_list_in_parent_mut(proj, new_parent).unwrap().pop().unwrap();
            Self::get_list_in_parent_mut(proj, init_parent).unwrap().insert(idx, obj_box);
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            *obj.parent_mut() = init_parent;
        };

        redo(project);

        Some(ObjAction::new(redo, undo))
    } 

    fn set_index(project: &mut Project, obj_ptr: ObjPtr<Self>, new_idx: usize) -> Option<ObjAction> {
        let obj = Self::get_list_mut(project).get_mut(obj_ptr)?;
        let parent = *obj.parent_mut();
        let sibling_list = Self::get_list_in_parent_mut(project, parent)?;
        let old_idx = sibling_list.iter().position(|other_obj| other_obj.make_ptr() == obj_ptr)?;
        let mut new_idx = new_idx.clamp(0, sibling_list.len());
        if new_idx > old_idx {
            new_idx -= 1;
        }

        let redo = move |proj: &'_ mut Project| {
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            let parent = *obj.parent_mut();
            let sibling_list = Self::get_list_in_parent_mut(proj, parent).unwrap();
            let obj = sibling_list.remove(old_idx);
            sibling_list.insert(new_idx, obj);
        };

        let undo = move |proj: &'_ mut Project| {
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            let parent = *obj.parent_mut();
            let sibling_list = Self::get_list_in_parent_mut(proj, parent).unwrap();
            let obj = sibling_list.remove(new_idx);
            sibling_list.insert(old_idx, obj);
        };

        redo(project);

        Some(ObjAction::new(redo, undo)) 
    }

}

pub trait HasRootAsset: Obj {

    type RootAsset: Asset + Obj<ListType = AssetList<Self::RootAsset>>;
    fn get_root_asset(project: &Project, ptr: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>>;

}
