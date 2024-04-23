
use project_macros::{ObjClone, ObjSerialize, Object};
use unique_type_id::UniqueTypeId;
use super::{action::ObjAction, folder::Folder, obj::{asset::Asset, child_obj::{ChildObj, HasRootAsset}, Obj, ObjBox, ObjClone, ObjPtr, ObjSerialize}, AssetPtr, Project};
use crate::project::obj::obj_list::ObjListTrait;

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
pub struct PaletteColor {
    #[field]
    pub color: glam::Vec4,
    #[parent]
    pub palette: ObjPtr<Palette>
}

impl ChildObj for PaletteColor {
    type Parent = ObjPtr<Palette>;

    fn parent(&self) -> Self::Parent {
        self.palette
    }

    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.palette
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        Some(&project.palettes.get(parent)?.colors)
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        Some(&mut project.palettes.get_mut(parent)?.colors)
    }

}

impl HasRootAsset for PaletteColor {

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

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
#[asset]
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
    type Parent = ObjPtr<Folder>;

    fn parent(&self) -> Self::Parent {
        self.folder
    }

    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.folder
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        Some(&project.folders.get(parent)?.palettes)
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        Some(&mut project.folders.get_mut(parent)?.palettes)
    }

}

impl HasRootAsset for Palette {

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

    fn extension() -> &'static str {
        "cippal"
    }

    fn type_magic_bytes() -> [u8; 4] {
        *b"palt" 
    }
    
    fn make_asset_ptr(ptr: ObjPtr<Self>) -> AssetPtr {
        AssetPtr::Palette(ptr)
    }

    fn icon() -> &'static str {
        egui_phosphor::regular::PALETTE
    }

}
