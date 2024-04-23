
use crate::project::{frame::Frame, obj::{Obj, ObjBox, ObjPtr}, sound_instance::SoundInstance, stroke::Stroke, Project};

use super::selection::Selection;

use crate::project::obj::obj_list::ObjListTrait;

pub enum Clipboard {
    None,
    Scene(Vec<ObjBox<Stroke>>),
    Timeline(Vec<ObjBox<Frame>>, Vec<ObjBox<SoundInstance>>) 
}

fn make_clipboard_copy<T: Obj>(project: &mut Project, selection_list: &Vec<ObjPtr<T>>) -> Vec<ObjBox<T>> {
    let mut clipboard_copy = Vec::new();
    for selected in selection_list {
        if let Some(clone) = selected.make_obj_clone(project) {
            clipboard_copy.push(T::get_list_mut(project).add(clone));
        }
    }
    clipboard_copy
}

impl Clipboard {

    pub fn from_selection(selection: &Selection, project: &mut Project) -> Self {
        match selection {
            Selection::None => Self::None, 
            Selection::Scene(strokes) => Clipboard::Scene(make_clipboard_copy(project, strokes)),
            Selection::Timeline(frames, sounds) => Clipboard::Timeline(make_clipboard_copy(project, frames), make_clipboard_copy(project, sounds)),
        }
    }

}
