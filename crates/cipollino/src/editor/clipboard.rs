
use crate::project::{frame::Frame, obj::ObjBox, stroke::Stroke, Project};

use super::selection::Selection;

pub enum Clipboard {
    None,
    Scene(Vec<ObjBox<Stroke>>),
    Frames(Vec<ObjBox<Frame>>) 
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
            Selection::Frames(frames) => {
                let mut clip_frames = Vec::new();
                for frame_ptr in frames {
                    if let Some(clone) = frame_ptr.make_obj_clone(project) {
                        clip_frames.push(project.frames.add(clone));
                    }
                }
                Self::Frames(clip_frames)
            },
        }
    }

}
