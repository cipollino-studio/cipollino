
use project_macros::{ObjClone, ObjSerialize, Object};
use unique_type_id::UniqueTypeId;

use super::{action::ObjAction, folder::Folder, layer::Layer, obj::{asset::Asset, child_obj::{ChildObj, HasRootAsset}, Obj, ObjBox, ObjClone, ObjList, ObjPtr, ObjSerialize}, AssetPtr, Project};

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
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
    type Parent = ObjPtr<Folder>;

    fn parent(&self) -> Self::Parent {
        self.folder
    }

    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.folder
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        Some(&project.folders.get(parent)?.graphics) 
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        Some(&mut project.folders.get_mut(parent)?.graphics) 
    }

}

impl HasRootAsset for Graphic {

    type RootAsset = Graphic;
    fn get_root_asset(_project: &Project, graphic: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        Some(graphic)
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

    fn make_asset_ptr(ptr: ObjPtr<Self>) -> AssetPtr {
        AssetPtr::Graphic(ptr)
    }

    fn icon() -> &'static str {
        egui_phosphor::regular::IMAGE_SQUARE
    }

}
