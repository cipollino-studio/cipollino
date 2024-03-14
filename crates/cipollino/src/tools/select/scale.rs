
use glam::{vec3, Mat4, Vec2};

use crate::{editor::state::EditorState, panels::scene::ScenePanel, util::geo::vec2_to_vec3};

use super::{FreeTransformPoints, Select, SelectState};

#[derive(Clone, Copy)]
pub enum ScalePivot {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft
}

pub struct Scale;

impl Scale {

    pub fn mouse_down(mouse_pos: Vec2, state: &mut EditorState, select: &mut Select, scene: &ScenePanel, pivot: ScalePivot) {
        let FreeTransformPoints {
            bl,
            tl,
            br,
            tr,
            bl_rotate: _,
            tl_rotate: _,
            br_rotate: _,
            tr_rotate: _
        } = select.freetransform_points(scene.cam_size);

        let pivot = match pivot {
            ScalePivot::TopRight => tr,
            ScalePivot::TopLeft => tl,
            ScalePivot::BottomRight => br,
            ScalePivot::BottomLeft => bl,
        }; 
        let (scl, rot, trans) = select.trans.to_scale_rotation_translation();
        let unrotate = glam::Mat4::from_quat(rot).inverse();
        let prev_pivot_to_mouse = unrotate.transform_point3(vec2_to_vec3(select.prev_mouse_pos - pivot));
        let pivot_to_mouse = unrotate.transform_point3(vec2_to_vec3(mouse_pos - pivot));
        let new_scl = vec3(scl.x * pivot_to_mouse.x / prev_pivot_to_mouse.x, scl.y * pivot_to_mouse.y / prev_pivot_to_mouse.y, scl.z); 
        let new_trans_unpivoted = Mat4::from_scale_rotation_translation(new_scl, rot, trans);
        let new_trans = select.pivot_matrix(new_trans_unpivoted, select.untransform(pivot));
        select.apply_transformation(new_trans, state);
    }

    pub fn mouse_release(select: &mut Select, state: &mut EditorState) {
        select.state = SelectState::FreeTransform;
        if let Some(action) = std::mem::replace(&mut select.transform_action, None){
            state.actions.add(action);
        } 
    }

}
