
use std::{path::PathBuf, str::FromStr};

use crate::project::{action::ObjAction, folder::Folder, Project};

use super::{ChildObj, Obj, ObjBox, ObjPtr};

pub trait Asset : Obj + ChildObj<Parent = Folder> {

    fn name(&self) -> &String;
    fn name_mut(&mut self) -> &mut String;
    fn extension(&self) -> &str;
    fn folder(&self) -> ObjPtr<Folder>;
    fn folder_mut(&mut self) -> &mut ObjPtr<Folder>;

    fn path(&self, project: &Project) -> Option<PathBuf> {
        let folder = project.folders.get(self.folder().clone())?;
        let mut path = folder.path();
        path.push(PathBuf::from_str((self.name().clone() + "." + self.extension()).as_str()).unwrap());
        Some(path)
    }

    fn asset_add(project: &mut Project, folder: ObjPtr<Folder>, mut obj: Self) -> Option<(ObjPtr<Self>, Vec<ObjAction>)> {
        let valid_name = next_valid_name(project, obj.name(), Self::get_sibling_list(project, folder)?);
        *obj.name_mut() = valid_name;
        *obj.folder_mut() = folder;
        let path = obj.path(project)?;
        let (ptr, add_act) = Self::add(project, folder, obj)?;
        Some((ptr, vec![add_act, ObjAction::new(|_proj| {

        }, move |proj| {
            proj.files_to_delete.insert(path.clone());
        })]))
    }

    fn asset_delete(project: &mut Project, obj_ptr: ObjPtr<Self>) -> Option<Vec<ObjAction>> {
        let folder_ptr = Self::get_list_mut(project).get_mut(obj_ptr)?.folder();
        let obj = Self::get_list(project).get(obj_ptr)?; 
        let path = obj.path(project)?;
        project.files_to_delete.insert(path.clone());
        Some(vec![Self::delete(project, folder_ptr, obj_ptr)?, ObjAction::new(move |proj| {
            proj.files_to_delete.insert(path.clone());
        }, |_proj| {

        })])
    }

    fn rename(project: &mut Project, obj_ptr: ObjPtr<Self>, name: String) -> Option<ObjAction> {
        let folder_ptr = Self::get_list_mut(project).get_mut(obj_ptr)?.folder();
        let folder = project.folders.get(folder_ptr)?;
        let obj = Self::get_list(project).get(obj_ptr)?;
        let init_name = obj.name().clone();
        let new_name = next_valid_name(project, &name, Self::get_list_in_parent(folder));
        project.files_to_delete.insert(obj.path(project)?);

        let redo = move |proj: &'_ mut Project| {
            let obj = Self::get_list(proj).get(obj_ptr).unwrap();
            let path = obj.path(proj);
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            *obj.name_mut() = new_name.clone(); 
            if let Some(path) = path {
                proj.files_to_delete.insert(path);
            }
        };

        let undo = move |proj: &'_ mut Project| {
            let obj = Self::get_list(proj).get(obj_ptr).unwrap();
            let path = obj.path(proj);
            let obj = Self::get_list_mut(proj).get_mut(obj_ptr).unwrap();
            *obj.name_mut() = init_name.clone(); 
            if let Some(path) = path {
                proj.files_to_delete.insert(path);
            }
        };

        redo(project);

        Some(ObjAction::new(redo, undo))
    }

}

pub fn next_valid_name<T: Asset>(project: &Project, name: &String, assets: &Vec<ObjBox<T>>) -> String {
    let names = assets.iter().map(|asset| asset.get(project).name());

    if names.clone().position(|other_name| other_name == name).is_none() {
        return name.clone();
    }

    for i in 1.. {
        let potential_name = format!("{} ({})", name, i);
        if names.clone().position(|other_name| other_name == &potential_name).is_none() {
            return potential_name;
        }
    }

    "".to_owned()
}
