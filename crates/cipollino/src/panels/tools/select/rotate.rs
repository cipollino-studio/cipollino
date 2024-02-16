
use glam::{Mat4, Quat, Vec2};

use crate::editor::EditorState;

use super::{Select, SelectState};

pub struct Rotate;

impl Rotate {

    pub fn mouse_down(mouse_pos: Vec2, state: &mut EditorState, select: &mut Select) {
        let pivot = select.transform(select.pivot);
        let angle = -(mouse_pos - pivot).angle_between(select.prev_mouse_pos - pivot);
        let (scl, rot, trans) = select.trans.to_scale_rotation_translation();
        let new_angle = rot.to_euler(glam::EulerRot::XYZ).2 + angle;
        let new_rot = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, new_angle);
        let new_trans_unpivoted = Mat4::from_scale_rotation_translation(scl, new_rot, trans);
        let new_trans = select.pivot_matrix(new_trans_unpivoted, select.pivot); 
        select.apply_transformation(new_trans, state);
    }

    pub fn mouse_release(select: &mut Select, state: &mut EditorState) {
        select.state = SelectState::FreeTransform;
        if let Some(action) = std::mem::replace(&mut select.transform_action, None){
            state.actions.add(action);
        } 
    }

}
