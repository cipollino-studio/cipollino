
mod meshgen;

use std::sync::Arc;

use glow::{Context, HasContext};

use crate::project::Project;

use super::{shader::Shader, fb::Framebuffer};

pub struct SceneRenderer {
    line_shader: Shader
}

impl SceneRenderer {

    pub fn new(gl: &Arc<Context>) -> Self {
        Self {
            line_shader: Shader::new("
                #version 100\n 

                attribute vec2 aPos;

                uniform mat4 uTrans;
        
                void main() {
                    gl_Position = uTrans * vec4(aPos, 0.0, 1.0);                
                } 
            ", "
                #version 100\n 

                void main() {
                    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
                }
            ", gl)
        }
    }

    pub fn render(
        &mut self,

        fb: &mut Framebuffer,
        w: u32,
        h: u32,

        cam_pos: glam::Vec2,
        cam_size: f32,

        project: &mut Project,
        gfx: u64,
        time: i32,

        gl: &Arc<Context>
    ) -> Option<()> {

        fb.resize(w, h, gl);
        fb.render_to(gl);

        unsafe {
            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }

        let aspect = (w as f32) / (h as f32);
        let proj = glam::Mat4::orthographic_rh_gl(-aspect * cam_size, aspect * cam_size, -cam_size, cam_size, -1.0, 1.0);
        let view = glam::Mat4::from_translation(-glam::vec3(cam_pos.x, cam_pos.y, 0.0));
        self.line_shader.enable(gl);
        self.line_shader.set_mat4("uTrans", &(proj * view), gl);

        let mut stroke_keys = Vec::new();
        for layer in &project.graphics.get(&gfx)?.layers {
            if let Some(frame) = project.get_frame_at(*layer, time) {
                stroke_keys.append(&mut project.frames.get(&frame)?.strokes.clone());
            }
        }
        for key in stroke_keys {
            if let Some(mesh) = meshgen::get_mesh(project, key, gl) {
                mesh.render(gl);
            }
        }

        None
        
    }

}
