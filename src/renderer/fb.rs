use std::sync::Arc;

use glow::{Context, HasContext};



pub struct Framebuffer {

    pub fbo: glow::NativeFramebuffer,
    pub color: glow::NativeTexture,
    pub w: u32,
    pub h: u32

}

impl Framebuffer {

    pub fn new(w: u32, h: u32, gl: &Arc<Context>) -> Framebuffer {
        unsafe {
            let fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let color = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(color));
            gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, w as i32, h as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(color), 0);
            assert!(gl.check_framebuffer_status(glow::FRAMEBUFFER) == glow::FRAMEBUFFER_COMPLETE);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            Self {
                fbo: fbo,
                color: color,
                w: w,
                h: h 
            }   
        }
    }

    pub fn resize(&mut self, w: u32, h: u32, gl: &Arc<Context>) {
        if w != self.w || h != self.h {
            self.delete(gl);
            *self = Self::new(w, h, gl);
        }
    }

    pub fn render_to(&self, gl: &Arc<Context>) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.w as i32, self.h as i32);
        }
    }

    pub fn render_to_win(w: u32, h: u32, gl: &Arc<Context>) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.viewport(0, 0, w as i32, h as i32);
        }
    }

    pub fn delete(&self, gl: &Arc<Context>) {
        unsafe {
            gl.delete_texture(self.color);
            gl.delete_framebuffer(self.fbo);
        }
    }

}
