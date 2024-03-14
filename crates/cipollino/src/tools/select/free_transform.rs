
use std::sync::Arc;

use glam::Vec2;

use crate::{editor::state::EditorState, panels::scene::{OverlayRenderer, ScenePanel}};

use super::{scale::ScalePivot, FreeTransformPoints, Select, SelectState};


pub struct FreeTransform;

impl FreeTransform {

    pub fn mouse_click(mouse_pos: Vec2, state: &mut EditorState, ui: &mut egui::Ui, select: &mut Select, scene: &mut ScenePanel, gl: &Arc<glow::Context>) {
        let FreeTransformPoints {
            bl,
            tl,
            br,
            tr,
            bl_rotate,
            tl_rotate,
            br_rotate,
            tr_rotate
        } = select.freetransform_points(scene.cam_size);
        let r = Self::overlay_circle_r(scene.cam_size);

        if (mouse_pos - tr).length() < r {
            select.state = SelectState::Scale(ScalePivot::BottomLeft);
            return;
        }
        if (mouse_pos - tl).length() < r {
            select.state = SelectState::Scale(ScalePivot::BottomRight);
            return;
        }
        if (mouse_pos - br).length() < r {
            select.state = SelectState::Scale(ScalePivot::TopLeft);
            return;
        }
        if (mouse_pos - bl).length() < r {
            select.state = SelectState::Scale(ScalePivot::TopRight);
            return;
        }

        if (mouse_pos - tr_rotate).length() < r || (mouse_pos - tl_rotate).length() < r || (mouse_pos - br_rotate).length() < r || (mouse_pos - bl_rotate).length() < r {
            select.state = SelectState::Rotate;
            return;
        }

        if let Some(stroke) = scene.sample_pick(mouse_pos, gl) {
            if state.selection.stroke_selected(stroke) {
                select.state = SelectState::Translate;
            } else {
                select.state = SelectState::Lasso;
                if !ui.input(|i| i.modifiers.shift) {
                    state.selection.clear();
                }
            }
            return;
        }

        select.state = SelectState::Lasso;
        if !ui.input(|i| i.modifiers.shift) {
            state.selection.clear();
        }
    }

    pub fn overlay_circle_r(cam_size: f32) -> f32 {
        0.025 * cam_size
    }

    pub fn draw_overlay(overlay: &mut OverlayRenderer, select: &mut Select) {
        let FreeTransformPoints {
            bl,
            tl,
            br,
            tr,
            bl_rotate,
            tl_rotate,
            br_rotate,
            tr_rotate
        } = select.freetransform_points(overlay.cam_size);
        
        let color = glam::vec4(1.0, 0.0, 0.0, 1.0);
        let r = Self::overlay_circle_r(overlay.cam_size);

        overlay.line(bl, tl, color);
        overlay.line(tl, tr, color);
        overlay.line(tr, br, color);
        overlay.line(br, bl, color);
        overlay.circle(select.transform(select.pivot), color, r);

        overlay.circle(tr, color, r);
        overlay.circle(tl, color, r);
        overlay.circle(br, color, r);
        overlay.circle(bl, color, r);

        overlay.circle(tr_rotate, color, r);
        overlay.circle(tl_rotate, color, r);
        overlay.circle(br_rotate, color, r);
        overlay.circle(bl_rotate, color, r);
        
    }

    pub fn mouse_cursor(mouse_pos: Vec2, scene: &mut ScenePanel, gl: &Arc<glow::Context>, select: &mut Select) -> egui::CursorIcon {
        let FreeTransformPoints {
            bl: _,
            tl: _,
            br: _,
            tr: _,
            bl_rotate,
            tl_rotate,
            br_rotate,
            tr_rotate
        } = select.freetransform_points(scene.cam_size);
        let r = Self::overlay_circle_r(scene.cam_size);

        if (mouse_pos - tr_rotate).length() < r || (mouse_pos - tl_rotate).length() < r || (mouse_pos - br_rotate).length() < r || (mouse_pos - bl_rotate).length() < r {
            return egui::CursorIcon::Alias;
        }
        if let Some(_stroke_key) = scene.sample_pick(mouse_pos, gl) {
            return egui::CursorIcon::Move;
        }
        egui::CursorIcon::Default
    }

}
