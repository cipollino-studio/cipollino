
use std::path::PathBuf;
use project_macros::{ObjClone, ObjSerialize, Object};
use unique_type_id::UniqueTypeId;
use crate::project::obj::{Obj, ObjList};
use crate::project::Project;
use super::file::audio::AudioFile;
use super::file::FilePtr;
use super::graphic::Graphic;
use super::obj::asset::Asset;
use super::obj::child_obj::{ChildObj, HasRootAsset};
use super::obj::{ObjBox, ObjClone, ObjPtr};
use super::action::ObjAction;
use super::palette::Palette;
use super::AssetPtr;
use super::obj::ObjSerialize;

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
pub struct Folder {
    #[field]
    pub name: String,

    #[parent]
    pub folder: ObjPtr<Folder>,
    pub graphics: Vec<ObjBox<Graphic>>,
    pub palettes: Vec<ObjBox<Palette>>,
    pub audios: Vec<FilePtr<AudioFile>>,
    pub folders: Vec<ObjBox<Folder>>,
}

impl ChildObj for Folder {
    type Parent = ObjPtr<Folder>;

    fn parent(&self) -> Self::Parent {
        self.folder
    }

    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.folder
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        Some(&project.folders.get(parent)?.folders)
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        Some(&mut project.folders.get_mut(parent)?.folders)
    }
}

impl HasRootAsset for Folder {

    type RootAsset = Folder;
    fn get_root_asset(_project: &Project, _folder: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        panic!("should never be called");
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

    fn make_asset_ptr(ptr: ObjPtr<Self>) -> AssetPtr {
        AssetPtr::Folder(ptr)
    }

    fn icon() -> &'static str {
        egui_phosphor::regular::FOLDER
    }

}

impl Folder {

    pub fn new(parent: ObjPtr<Folder>) -> Self {
        Self {
            name: "Folder".to_owned(),
            folder: parent,
            graphics: Vec::new(),
            palettes: Vec::new(),
            audios: Vec::new(),
            folders: Vec::new()
        }
    }

    pub fn file_path(&self, project: &Project) -> Option<PathBuf> {
        if let Some(parent) = project.folders.get(self.folder) {
            let mut path = parent.file_path(project)?;
            path.push(self.name.as_str());
            Some(path)
        } else {
            let mut path = project.save_path.clone();
            path.pop();
            Some(path)
        }
    }

    fn inside(project: &Project, folder: ObjPtr<Folder>, parent_ptr: ObjPtr<Folder>) -> bool {
        if let Some(parent) = project.folders.get(parent_ptr) {
            if parent.folder == ObjPtr::null() {
                return false;
            }
            if folder == parent_ptr {
                return true;
            }
            Self::inside(project, folder, parent.folder)
        } else {
            false
        }
    }

    pub fn asset_transfer(project: &mut Project, folder: ObjPtr<Folder>, new_folder: ObjPtr<Folder>) -> Option<Vec<ObjAction>> {
        if Self::inside(&project, new_folder, folder) {
            None
        } else {
            <Self as Asset>::asset_transfer(project, folder, new_folder)
        }
    }

}

impl Default for Folder {
    fn default() -> Self {
        Self::new(ObjPtr::null())
    }
}
