
use crate::project::{frame::Frame, obj::ObjBox, sound_instance::SoundInstance, stroke::Stroke, Project};

use super::selection::Selection;

pub enum Clipboard {
    None,
    Scene(Vec<ObjBox<Stroke>>),
    Frames(Vec<ObjBox<Frame>>, Vec<ObjBox<SoundInstance>>) 
}

impl Clipboard {

    pub fn from_selection(selection: &Selection, project: &mut Project) -> Self {
        match selection {
            Selection::None => Self::None, 
            Selection::Scene(strokes) => {
                let mut clip_strokes = Vec::new();
                for stroke_ptr in strokes {
                    if let Some(clone) = stroke_ptr.make_obj_clone(project) {
                        clip_strokes.push(project.strokes.add(clone));
                    } 
                }
                Self::Scene(clip_strokes)
            },
            Selection::Timeline(frames, sounds) => {
                let mut clip_frames = Vec::new();
                let mut clip_sounds = Vec::new();
                for frame_ptr in frames {
                    if let Some(clone) = frame_ptr.make_obj_clone(project) {
                        clip_frames.push(project.frames.add(clone));
                    }
                }
                for sound_ptr in sounds {
                    if let Some(clone) = sound_ptr.make_obj_clone(project) {
                        clip_sounds.push(project.sound_instances.add(clone));
                    }
                }
                Self::Frames(clip_frames, clip_sounds)
            },
        }
    }

}
