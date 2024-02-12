
use project_macros::{ObjClone, ObjSerialize, Object};

use super::{action::ObjAction, folder::Folder, layer::Layer, obj::{asset::Asset, child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjList, ObjPtr, ObjSerialize, ObjPtrAny}, Project};

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct Graphic {
    #[field]
    pub name: String,
    #[field]
    pub len: u32,
    #[field]
    pub clip: bool,
    #[field]
    pub w: u32,
    #[field]
    pub h: u32,
    pub layers: Vec<ObjBox<Layer>>,
    #[parent]
    pub folder: ObjPtr<Folder>
}

impl Default for Graphic {

    fn default() -> Self {
        Self {
            name: "Graphic".to_owned(),
            len: 100,
            clip: false,
            w: 1920,
            h: 1080,
            layers: Vec::new(),
            folder: ObjPtr::null()
        }
    }
}

impl ChildObj for Graphic {
    type Parent = Folder;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.folder
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.graphics
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.graphics
    }


}

impl Asset for Graphic {

    fn name(&self) -> &String {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn extension(&self) -> &str {
        "cipgfx"
    }

    fn folder(&self) -> ObjPtr<Folder> {
        self.folder
    }

    fn folder_mut(&mut self) -> &mut ObjPtr<Folder> {
        &mut self.folder
    }

}
