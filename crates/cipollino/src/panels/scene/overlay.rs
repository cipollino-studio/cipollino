
use glam::Vec2;

use crate::{editor::{selection::Selection, EditorSystems, state::EditorState}, project::{graphic::Graphic, obj::ObjPtr}};
use super::ScenePanel;
use glow::HasContext;

pub struct OverlayRenderer<'a, 'b> {
    renderer: &'a mut EditorSystems<'b>,
    proj_view: glam::Mat4,
    pub cam_size: f32,
}

impl<'a, 'b> OverlayRenderer<'a, 'b> {

    pub fn new(renderer: &'a mut EditorSystems<'b>, proj_view: glam::Mat4, cam_size: f32) -> Self {
        Self {
            renderer,
            proj_view,
            cam_size,
        }
    }

    pub fn line(&mut self, p0: glam::Vec2, p1: glam::Vec2, color: glam::Vec4) {
        let center = (p0 + p1) / 2.0;
        let len = (p1 - p0).length();
        let scale = glam::vec3(len, 0.004 * self.cam_size, 1.0);
        let angle = glam::vec2(1.0, 0.0).angle_between(p1 - p0);
        let model = glam::Mat4::from_translation(glam::vec3(center.x, center.y, 0.0)) * glam::Mat4::from_axis_angle(glam::vec3(0.0, 0.0, 1.0), angle) * glam::Mat4::from_scale(scale);
        let trans = self.proj_view * model; 
        self.renderer.renderer.flat_color_shader.enable(self.renderer.gl);
        self.renderer.renderer.flat_color_shader.set_mat4("uTrans", &trans, self.renderer.gl);
        self.renderer.renderer.flat_color_shader.set_vec4("uColor", color, self.renderer.gl);
        self.renderer.renderer.quad.render(self.renderer.gl);
    }

    pub fn circle(&mut self, pt: Vec2, color: glam::Vec4, r: f32) {
        let model = glam::Mat4::from_translation(glam::vec3(pt.x, pt.y, 0.0)) * glam::Mat4::from_scale(glam::Vec3::splat(r * 2.0));
        let trans = self.proj_view * model;
        self.renderer.renderer.circle_shader.enable(self.renderer.gl);
        self.renderer.renderer.flat_color_shader.set_mat4("uTrans", &trans, self.renderer.gl);
        self.renderer.renderer.flat_color_shader.set_vec4("uColor", color, self.renderer.gl);
        self.renderer.renderer.quad.render(self.renderer.gl);
    }

}

impl ScenePanel {

    pub fn render_overlays(&self, gfx: ObjPtr<Graphic>, systems: &mut EditorSystems, proj_view: glam::Mat4, state: &mut EditorState) {
        unsafe {
            systems.gl.disable(glow::DEPTH_TEST);
        }

        if let Some(gfx) = state.project.graphics.get(gfx) { 

            // Clip shadow
            if gfx.clip {
                systems.renderer.clip_shadow_shader.enable(systems.gl);
                let trans = proj_view * glam::Mat4::from_scale(glam::vec3(gfx.w as f32, gfx.h as f32, 1.0)); 
                systems.renderer.clip_shadow_shader.set_mat4("uTrans", &trans, systems.gl);
                systems.renderer.clip_shadow_mesh.render(systems.gl);
            }

            let mut overlay = OverlayRenderer::new(systems, proj_view, self.cam_size);

            state.curr_tool.clone().write().unwrap().draw_overlay(&mut overlay, state);

            if let Selection::Scene(strokes) = &state.selection {
                for stroke in strokes {
                    if let Some(stroke) = state.project.strokes.get(*stroke) {
                        for segment in stroke.iter_bezier_segments() {
                            let pts = segment.to_discrete::<8>();
                            for pts in pts.windows(2) {
                                overlay.line(pts[0], pts[1], glam::vec4(0.0, 1.0, 1.0, 1.0)); 
                            }
                        }
                    }
                }
            }
        }
    }

}
