
use project_macros::{ObjClone, ObjSerialize, Object};
use super::{action::ObjAction, folder::Folder, obj::{asset::Asset, child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjList, ObjPtr, ObjPtrAny, ObjSerialize}, Project, AssetPtr};

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct PaletteColor {
    #[field]
    pub color: glam::Vec4,
    #[parent]
    pub palette: ObjPtr<Palette>
}

impl ChildObj for PaletteColor {
    type Parent = Palette;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.palette
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.colors
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.colors
    }

    type RootAsset = Palette;
    fn get_root_asset(project: &Project, palette_color: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        Some(project.palette_colors.get(palette_color)?.palette)
    }

}

impl Default for PaletteColor {

    fn default() -> Self {
        Self {
            color: glam::vec4(0.0, 0.0, 0.0, 1.0),
            palette: ObjPtr::null()
        }
    }

}

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct Palette {
    #[field]
    pub name: String,
    pub colors: Vec<ObjBox<PaletteColor>>,
    #[parent]
    pub folder: ObjPtr<Folder>
}

impl Palette {

    pub fn new(folder: ObjPtr<Folder>) -> Self {
        Self {
            name: "Palette".to_owned(),
            colors: Vec::new(),
            folder,
        }
    }

}

impl Default for Palette {

    fn default() -> Self {
        Self::new(ObjPtr::null()) 
    }

}

impl ChildObj for Palette {
    type Parent = Folder;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.folder
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.palettes
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.palettes
    }

    type RootAsset = Palette;
    fn get_root_asset(_project: &Project, palette: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        Some(palette)
    }

}

impl Asset for Palette {

    fn name(&self) -> &String {
        &self.name
    }

    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    fn extension(&self) -> &str {
        "cippal"
    }

    fn folder(&self) -> ObjPtr<Folder> {
        self.folder
    }

    fn folder_mut(&mut self) -> &mut ObjPtr<Folder> {
        &mut self.folder 
    }
    
    fn make_asset_ptr(ptr: ObjPtr<Self>) -> AssetPtr {
        AssetPtr::Palette(ptr)
    }

    fn icon() -> &'static str {
        egui_phosphor::regular::PALETTE
    }

}
