
use project_macros::{ObjClone, ObjSerialize, Object};
use super::{action::ObjAction, file::{audio::AudioFile, FilePtr}, layer::Layer, obj::{child_obj::ChildObj, Obj, ObjBox, ObjClone, ObjList, ObjPtr, ObjPtrAny, ObjSerialize}, Project};

#[derive(Object, Clone, ObjClone, ObjSerialize)]
pub struct SoundInstance {    
    #[parent]
    pub layer: ObjPtr<Layer>,
    // Begin and end are measured in samples
    #[field]
    pub begin: i64,
    #[field]
    pub end: i64,
    pub audio: FilePtr<AudioFile>
}

impl ChildObj for SoundInstance {
    type Parent = Layer;

    fn parent_mut(&mut self) -> &mut ObjPtr<Self::Parent> {
        &mut self.layer
    }

    fn get_list_in_parent(parent: &Self::Parent) -> &Vec<ObjBox<Self>> {
        &parent.sound_instances
    }

    fn get_list_in_parent_mut(parent: &mut Self::Parent) -> &mut Vec<ObjBox<Self>> {
        &mut parent.sound_instances
    }
}

impl Default for SoundInstance {

    fn default() -> Self {
        Self {
            layer: ObjPtr::null(),
            begin: 0,
            end: 0,
            audio: FilePtr::null()
        }
    }

}
