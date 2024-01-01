
mod meshgen;

use std::sync::Arc;

use glow::{Context, HasContext};

use crate::project::Project;

use super::{shader::Shader, fb::Framebuffer, mesh::Mesh};

pub struct SceneRenderer {
    pub flat_color_shader: Shader,
    pub circle_shader: Shader,
    pub quad: Mesh
}

impl SceneRenderer {

    pub fn new(gl: &Arc<Context>) -> Self {
        let mut quad = Mesh::new(vec![2], gl);
        quad.upload(&vec![
            -0.5, -0.5,
             0.5, -0.5,
            -0.5,  0.5,
             0.5,  0.5
        ], &vec![
            0, 1, 2,
            1, 2, 3
        ], gl);
        Self {
            flat_color_shader: Shader::new("
                #version 100\n 

                attribute vec2 aPos;

                uniform mat4 uTrans;
        
                void main() {
                    gl_Position = uTrans * vec4(aPos, 0.0, 1.0);                
                } 
            ", "
                #version 100\n 

                uniform highp vec4 uColor;

                void main() {
                    gl_FragColor = uColor;
                }
            ", gl),
            circle_shader: Shader::new("
                #version 100\n 

                attribute vec2 aPos;

                uniform mat4 uTrans;

                varying vec2 pUv;
        
                void main() {
                    gl_Position = uTrans * vec4(aPos, 0.0, 1.0);                
                    pUv = aPos;
                } 
            ", "
                #version 100\n 

                uniform highp vec4 uColor;

                varying mediump vec2 pUv;

                void main() {
                    gl_FragColor = uColor;
                    if(length(pUv) > 0.5) {
                        gl_FragColor = vec4(0.0);
                    }
                }
            ", gl),
            quad
        }
    }

    pub fn render(
        &mut self,

        fb: &mut Framebuffer,
        fb_pick: Option<(&mut Framebuffer, &mut Vec<u64>)>,
        w: u32,
        h: u32,

        cam_pos: glam::Vec2,
        cam_size: f32,

        project: &mut Project,
        gfx: u64,
        time: i32,

        onion_before: i32,
        onion_after: i32,

        gl: &Arc<Context>
    ) -> Option<glam::Mat4> {

        fb.resize(w, h, gl);
        fb.render_to(gl);

        unsafe {
            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.enable(glow::BLEND);
        }

        let aspect = (w as f32) / (h as f32);
        let proj = glam::Mat4::orthographic_rh_gl(-aspect * cam_size, aspect * cam_size, -cam_size, cam_size, -1.0, 1.0);
        let view = glam::Mat4::from_translation(-glam::vec3(cam_pos.x, cam_pos.y, 0.0));
        let proj_view = proj * view;
        self.flat_color_shader.enable(gl);
        self.flat_color_shader.set_mat4("uTrans", &proj_view, gl);

        let mut onion_strokes = Vec::new();
        let mut stroke_keys = Vec::new();
        for layer in &project.graphics.get(&gfx)?.layers {
            if let Some(frame) = project.get_frame_at(*layer, time) {
                let frame = project.frames.get(&frame)?;
                let mut curr_time = frame.data.time;
                let mut alpha = 0.75;
                for _i in 0..onion_before {
                    if let Some(frame) = project.get_frame_before(*layer, curr_time) {
                        let frame = project.frames.get(&frame)?;
                        onion_strokes.append(&mut (frame.strokes.clone().iter().map(|key| (glam::vec4(1.0, 0.3, 1.0, alpha), *key)).collect()));
                        alpha *= 0.8;
                        curr_time = frame.data.time;
                    }
                }
                // Ugly bug fix: make sure the oldest strokes are drawn at the back
                onion_strokes.reverse();
                let mut curr_time = frame.data.time;
                let mut alpha = 0.75;
                for _i in 0..onion_after {
                    if let Some(frame) = project.get_frame_after(*layer, curr_time) {
                        let frame = project.frames.get(&frame)?;
                        onion_strokes.append(&mut (frame.strokes.clone().iter().map(|key| (glam::vec4(0.3, 1.0, 1.0, alpha), *key)).collect()));
                        alpha *= 0.8;
                        curr_time = frame.data.time;
                    }
                }
                stroke_keys.append(&mut frame.strokes.clone());
            }
        }
        for (color, key) in onion_strokes {
            if let Some(mesh) = meshgen::get_mesh(project, key, gl) {
                self.flat_color_shader.set_vec4("uColor", color, gl);
                mesh.render(gl);
            }
        }
        for key in &stroke_keys {
            let color = project.strokes.get(key)?.data.color;
            let key = *key;
            if let Some(mesh) = meshgen::get_mesh(project, key, gl) {
                self.flat_color_shader.set_vec4("uColor", glam::vec4(color.x, color.y, color.z, 1.0), gl);
                mesh.render(gl);
            }
        }

        if let Some((fb_pick, color_key_map)) = fb_pick {
            fb_pick.resize(w, h, gl);
            fb_pick.render_to(gl);
            color_key_map.clear();

            unsafe {
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
            for key in &stroke_keys {
                let key = *key;
                if let Some(mesh) = meshgen::get_mesh(project, key, gl) {
                    let mut color = 0 as u32;
                    for i in 0..color_key_map.len() {
                        if color_key_map[i] == key {
                            color = i as u32;
                            break;
                        } 
                    };
                    if color == 0 {
                        color_key_map.push(key);
                        color = color_key_map.len() as u32;
                    }
                    let bytes = color.to_le_bytes();
                    let r = (bytes[0] as f32) / 255.0;
                    let g = (bytes[1] as f32) / 255.0;
                    let b = (bytes[2] as f32) / 255.0;
                    self.flat_color_shader.set_vec4("uColor", glam::vec4(r, g, b, 1.0), gl);
                    mesh.render(gl);
                }
            }
            fb.render_to(gl);
        }

        Some(proj_view) 
        
    }

}
