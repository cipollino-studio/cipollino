
use std::path::PathBuf;

use project_macros::{ObjClone, Object};
use crate::project::obj::{Obj, ObjList};
use crate::project::Project;
use super::graphic::Graphic;
use super::obj::asset::Asset;
use super::obj::child_obj::ChildObj;
use super::obj::{ObjBox, ObjClone, ObjPtr};
use super::action::ObjAction;


#[derive(Object, Clone, ObjClone)]
pub struct Folder {
    #[field]
    pub name: String,

    pub folder: ObjPtr<Folder>,
    pub graphics: Vec<ObjBox<Graphic>>,
    pub folders: Vec<ObjBox<Folder>>
}

impl ChildObj for Folder {
    type Parent = Folder;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.folder
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.folders
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.folders
    }
}

impl Asset for Folder {

    fn name(&self) -> &String {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn extension(&self) -> &str {
        ""
    }

    fn folder(&self) -> ObjPtr<Folder> {
        self.folder
    }

    fn folder_mut(&mut self) -> &mut ObjPtr<Folder> {
        &mut self.folder
    }
}

impl Folder {

    pub fn new(parent: ObjPtr<Folder>) -> Self {
        Self {
            name: "Folder".to_owned(),
            folder: parent,
            graphics: Vec::new(),
            folders: Vec::new()
        }
    }

    pub fn path(&self, project: &Project) -> Option<PathBuf> {
        if let Some(parent) = project.folders.get(self.folder) {
            let mut path = parent.path(project)?;
            path.push(self.name.as_str());
            Some(path)
        } else {
            let mut path = project.save_path.clone()?;
            path.pop();
            Some(path) 
        }
    }



}

impl Default for Folder {
    fn default() -> Self {
        Self::new(ObjPtr::null())
    }
}
