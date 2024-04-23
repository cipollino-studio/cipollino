
use project_macros::{ObjClone, ObjSerialize, Object};
use unique_type_id::UniqueTypeId;
use super::{action::ObjAction, resource::{audio::AudioFile, ResPtr}, graphic::Graphic, layer::Layer, obj::{child_obj::{ChildObj, HasRootAsset}, Obj, ObjBox, ObjClone, ObjPtr, ObjSerialize}, Project};
use crate::project::obj::obj_list::ObjListTrait;

#[derive(Object, Clone, ObjClone, ObjSerialize, UniqueTypeId)]
pub struct SoundInstance {    
    #[parent]
    pub layer: ObjPtr<Layer>,
    // Begin and end are measured in samples
    #[field]
    pub begin: i64,
    #[field]
    pub end: i64,
    #[field]
    pub offset: i64,
    pub audio: ResPtr<AudioFile>
}

impl ChildObj for SoundInstance {
    type Parent = ObjPtr<Layer>;

    fn parent(&self) -> Self::Parent {
        self.layer
    }
    
    fn parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.layer
    }

    fn get_list_in_parent(project: &Project, parent: Self::Parent) -> Option<&Vec<ObjBox<Self>>> {
        Some(&project.layers.get(parent)?.sound_instances)
    }

    fn get_list_in_parent_mut(project: &mut Project, parent: Self::Parent) -> Option<&mut Vec<ObjBox<Self>>> {
        Some(&mut project.layers.get_mut(parent)?.sound_instances)
    }

}

impl HasRootAsset for SoundInstance {

    type RootAsset = Graphic;
    fn get_root_asset(project: &Project, sound_instance: ObjPtr<Self>) -> Option<ObjPtr<Self::RootAsset>> {
        Layer::get_root_asset(project, project.sound_instances.get(sound_instance)?.layer)
    }

}

impl Default for SoundInstance {

    fn default() -> Self {
        Self {
            layer: ObjPtr::null(),
            begin: 0,
            end: 0,
            offset: 0,
            audio: ResPtr::null()
        }
    }

}
