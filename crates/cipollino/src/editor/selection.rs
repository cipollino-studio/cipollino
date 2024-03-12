
use crate::project::{frame::Frame, obj::ObjPtr, sound_instance::SoundInstance, stroke::Stroke}; 

pub enum Selection {
    None,
    Scene(Vec<ObjPtr<Stroke>>),
    Timeline(Vec<ObjPtr<Frame>>, Vec<ObjPtr<SoundInstance>>)
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
        if let Self::Timeline(..) = self {
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
        if let Self::Timeline(frames, _) = self {
            frames.contains(&frame)
        } else {
            false
        }
    }
    
    pub fn sound_selected(&self, sound: ObjPtr<SoundInstance>) -> bool {
        if let Self::Timeline(_, sounds) = self {
            sounds.contains(&sound)
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
        if let Self::Timeline(frames, _) = self {
            if frames.iter().position(|other_frame| *other_frame == frame).is_none() {
                frames.push(frame);
            }
        } else {
            let new_sel = Self::Timeline(vec![frame], vec![]);
            *self = new_sel;
        }
    }

    pub fn select_frame_inverting(&mut self, frame: ObjPtr<Frame>) {
        if let Self::Timeline(frames, ..) = self {
            if let Some(idx) = frames.iter().position(|other_frame| *other_frame == frame) {
                frames.remove(idx);
            } else {
                frames.push(frame);
            }
        } else {
            let new_sel = Self::Timeline(vec![frame], vec![]);
            *self = new_sel;
        }
    }

    pub fn select_sound(&mut self, sound: ObjPtr<SoundInstance>) {
        if let Self::Timeline(_, sounds) = self {
            if sounds.iter().position(|other_sound| *other_sound == sound).is_none() {
                sounds.push(sound);
            }
        } else {
            let new_sel = Self::Timeline(vec![], vec![sound]);
            *self = new_sel;
        }
    }

    pub fn select_sound_inverting(&mut self, sound: ObjPtr<SoundInstance>) {
        if let Self::Timeline(_, sounds) = self {
            if let Some(idx) = sounds.iter().position(|other_sound| *other_sound == sound) {
                sounds.remove(idx);
            } else {
                sounds.push(sound);
            }
        } else {
            let new_sel = Self::Timeline(vec![], vec![sound]);
            *self = new_sel;
        }
    }
    
}
