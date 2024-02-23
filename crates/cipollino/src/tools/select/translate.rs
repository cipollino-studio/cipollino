use glam::{Mat4, Vec2};

use crate::editor::EditorState;

use super::{Select, SelectState};


pub struct Translate;

impl Translate {

    pub fn mouse_down(mouse_pos: Vec2, state: &mut EditorState, select: &mut Select) {
        let delta = mouse_pos - select.prev_mouse_pos;
        let (scl, rot, trans) = select.trans.to_scale_rotation_translation();
        select.apply_transformation(Mat4::from_scale_rotation_translation(scl, rot, trans + glam::vec3(delta.x, delta.y, 0.0)), state);
    }

    pub fn mouse_release(select: &mut Select, state: &mut EditorState) {
        select.state = SelectState::FreeTransform;
        if let Some(action) = std::mem::replace(&mut select.transform_action, None){
            state.actions.add(action);
        } 
    }

}