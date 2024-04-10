
use std::{path::PathBuf, str::FromStr};

use crate::{project::{action::ObjAction, folder::Folder, AssetPtr, Project}, util};

use super::{child_obj::ChildObj, Obj, ObjBox, ObjPtr};


pub trait Asset : Obj + ChildObj<Parent = ObjPtr<Folder>> {

    fn name(&self) -> &String;
    fn name_mut(&mut self) -> &mut String;
    fn extension(&self) -> &str;

    fn folder(&self) -> ObjPtr<Folder> {
        self.parent()
    }

    fn folder_mut(&mut self) -> &mut ObjPtr<Folder> {
        self.parent_mut()
    }

    fn make_asset_ptr(ptr: ObjPtr<Self>) -> AssetPtr;
    
    fn icon() -> &'static str;

    fn file_path(&self, project: &Project) -> Option<PathBuf> {
        let folder = project.folders.get(self.folder().clone())?;
        let mut path = folder.file_path(project)?;
        let extension = if self.extension().len() == 0 { "".to_owned() } else { ".".to_owned() + self.extension() };
        path.push(PathBuf::from_str((self.name().clone() + extension.as_str()).as_str()).unwrap());
        Some(path)
    }

    fn asset_add(project: &mut Project, folder: ObjPtr<Folder>, mut obj: Self) -> Option<(ObjPtr<Self>, Vec<ObjAction>)> {
        let valid_name = next_valid_name(project, obj.name(), Self::get_list_in_parent(project, folder)?);
        *obj.name_mut() = valid_name;
        *obj.folder_mut() = folder;
        let (ptr, add_act) = Self::add(project, folder, obj)?;

        Some((ptr, vec![add_act, ObjAction::new(|_proj| {

        }, move |proj| {
            if let Some(obj) = Self::get_list(proj).get(ptr) {
                if let Some(path) = obj.file_path(proj) {
                    util::fs::remove(&path);
                }
            }
        })]))
    }

    fn asset_delete(project: &mut Project, obj_ptr: ObjPtr<Self>) -> Option<Vec<ObjAction>> {
        let obj = Self::get_list(project).get(obj_ptr)?; 
        if let Some(path) = obj.file_path(project) {
            util::fs::remove(&path);
        }
        Some(vec![Self::delete(project, obj_ptr)?, ObjAction::new(move |proj| {
            if let Some(obj) = Self::get_list(proj).get(obj_ptr) { 
                if let Some(path) = obj.file_path(proj) {
                    util::fs::remove(&path);
                }
            }
        }, |_proj| {

        })])
    }

    fn rename(project: &mut Project, obj_ptr: ObjPtr<Self>, name: String) -> Option<ObjAction> {
        let folder_ptr = Self::get_list_mut(project).get_mut(obj_ptr)?.folder();
        let obj = Self::get_list(project).get(obj_ptr)?;
        let init_name = obj.name().clone();
        let new_name = next_valid_name(project, &name, Self::get_list_in_parent(&project, folder_ptr).unwrap());

        let redo = move |proj: &'_ mut Project| {
            let obj = Self::get_list(proj).get(obj_ptr).unwrap();
            let path = obj.file_path(proj);
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            *obj.name_mut() = new_name.clone(); 
            if let Some(path) = path {
                util::fs::remove(&path);
            }
        };

        let undo = move |proj: &'_ mut Project| {
            let obj = Self::get_list(proj).get(obj_ptr).unwrap();
            let path = obj.file_path(proj);
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            *obj.name_mut() = init_name.clone(); 
            if let Some(path) = path {
                util::fs::remove(&path);
            }
        };

        redo(project);

        Some(ObjAction::new(redo, undo))
    }

    fn asset_transfer(project: &mut Project, asset: ObjPtr<Self>, new_folder: ObjPtr<Folder>) -> Option<Vec<ObjAction>> {
        let obj = Self::get_list(project).get(asset)?;
        if obj.folder() == new_folder {
            return None;
        }
        let init_name = obj.name().clone();
        let new_name = next_valid_name(project, &init_name, Self::get_list_in_parent(project, new_folder)?);
        if let Some(path) = obj.file_path(project) {
            util::fs::remove(&path);
        }
        let obj = Self::get_list_mut(project).get_mut(asset)?;
        *obj.name_mut() = new_name.clone();
        
        Some(vec![ObjAction::new(move |proj| {
            let obj = Self::get_list(proj).get(asset).unwrap();
            if let Some(path) = obj.file_path(proj) {
                util::fs::remove(&path);
            }
            let obj = Self::get_list_mut(proj).get_mut(asset).unwrap();
            *obj.name_mut() = new_name.clone();
        }, |_| {}), Self::transfer(project, asset, new_folder)?, ObjAction::new(|_| {}, move |proj| {
            let obj = Self::get_list(proj).get(asset).unwrap();
            if let Some(path) = obj.file_path(proj) {
                util::fs::remove(&path);
            }
            let obj = Self::get_list_mut(proj).get_mut(asset).unwrap();
            *obj.name_mut() = init_name.clone();
        })])
    }

    fn find_by_name<'a>(project: &'a Project, folder: ObjPtr<Folder>, name: &str) -> Option<&'a ObjBox<Self>> {
        for other in Self::get_list_in_parent(project, folder)? {
            if other.get(project).name() == name {
                return Some(other);
            }
        }
        None
    }

}

pub fn next_valid_name<T: Asset>(project: &Project, name: &String, assets: &Vec<ObjBox<T>>) -> String {
    let names = assets.iter().map(|asset| asset.get(project).name().as_str());
    util::next_unique_name(name, names)
}
