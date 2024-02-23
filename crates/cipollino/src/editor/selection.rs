
use crate::project::{obj::ObjPtr, stroke::Stroke, frame::Frame}; 

pub enum Selection {
    None,
    Scene(Vec<ObjPtr<Stroke>>),
    Frames(Vec<ObjPtr<Frame>>)
}

impl Selection {

    pub fn clear(&mut self) {
        *self = Self::None;
    }

    pub fn is_scene(&self) -> bool {
        if let Self::Scene(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_frames(&self) -> bool {
        if let Self::Frames(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Selection::None => true,
            _ => false
        }
    }

    pub fn stroke_selected(&self, stroke: ObjPtr<Stroke>) -> bool {
        if let Self::Scene(strokes) = self {
            strokes.contains(&stroke)
        } else {
            false
        }
    }

    pub fn frame_selected(&self, frame: ObjPtr<Frame>) -> bool {
        if let Self::Frames(frames) = self {
            frames.contains(&frame)
        } else {
            false
        }
    }

    pub fn select_stroke_inverting(&mut self, stroke: ObjPtr<Stroke>) {
        if let Self::Scene(strokes) = self {
            if let Some(idx) = strokes.iter().position(|other_stroke| *other_stroke == stroke) {
                strokes.remove(idx);
            } else {
                strokes.push(stroke);
            }
        } else {
            let new_sel = Self::Scene(vec![stroke]);
            *self = new_sel;
        }
    }

    pub fn select_frame(&mut self, frame: ObjPtr<Frame>) {
        if let Self::Frames(frames) = self {
            if frames.iter().position(|other_frame| *other_frame == frame).is_none() {
                frames.push(frame);
            }
        } else {
            let new_sel = Self::Frames(vec![frame]);
            *self = new_sel;
        }
    }

    pub fn select_frame_inverting(&mut self, frame: ObjPtr<Frame>) {
        if let Self::Frames(frames) = self {
            if let Some(idx) = frames.iter().position(|other_frame| *other_frame == frame) {
                frames.remove(idx);
            } else {
                frames.push(frame);
            }
        } else {
            let new_sel = Self::Frames(vec![frame]);
            *self = new_sel;
        }
    }
    
}
