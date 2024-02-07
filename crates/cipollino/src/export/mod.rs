
use std::path::PathBuf;

use glow::HasContext;

use crate::{renderer::fb::Framebuffer, editor::{EditorState, EditorRenderer}, project::{obj::ObjPtr, graphic::Graphic}};

pub struct Export {
    fb: Option<(Framebuffer, Framebuffer)>,
    pub dialog_open: bool,
    pub exporting: Option<(PathBuf, i32, ObjPtr<Graphic>)> 
}

impl Export {
    
    pub fn new() -> Self {
        Self {
            fb: None,
            dialog_open: false,
            exporting: None 
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut EditorState, renderer: &mut EditorRenderer) {
        let mut close_dialog = false;
        egui::Window::new("Export")
            .open(&mut self.dialog_open)
            .show(ctx, |ui| {
                let graphic_exists = state.project.graphics.get(state.open_graphic).is_some();
                if ui.add_enabled(graphic_exists, egui::Button::new("Export"))
                    .clicked() {
                    if let Some(mut path) = rfd::FileDialog::new().save_file() {
                        path.set_extension("png");
                        self.exporting = Some((path, 0, state.open_graphic));
                        close_dialog = true;
                    }
                }
            });
        if close_dialog {
            self.dialog_open = false;
        }
        
        renderer.use_renderer(|gl, renderer| {
            if let Some((path, frame, gfx_key)) = &mut self.exporting {
                let frame_copy = *frame;
                if let None = self.fb {
                    self.fb = Some((
                        Framebuffer::new(1920, 1080, gl),
                        Framebuffer::new(1920, 1080, gl)
                    ));
                } 
                if let Some((fb, aa_fb)) = self.fb.as_mut() {
                    let aa_scl = 2;
                    let gfx = state.project.graphics.get(*gfx_key).unwrap();
                    let w = gfx.w;
                    let h = gfx.h;
                    renderer.render(fb, None, w * aa_scl, h * aa_scl, glam::Vec2::ZERO, h as f32 / 2.0, &mut state.project, *gfx_key, *frame, 0, 0, gl);
                    aa_fb.resize(w, h, gl);
                    unsafe {
                        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(aa_fb.fbo));
                        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fb.fbo));
                        gl.blit_framebuffer(0, (h * aa_scl - 1) as i32, (w * aa_scl) as i32, 0, 0, 0, w as i32, h as i32, glow::COLOR_BUFFER_BIT, glow::LINEAR);
                    }
                    let mut pixel_data = Vec::new();
                    // TODO: this is probably inneficient
                    pixel_data.reserve((w * h * 3) as usize);
                    for _i in 0..(w * h * 3) {
                        pixel_data.push(0);
                    }
                    unsafe {
                        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(aa_fb.fbo));
                        gl.read_pixels(0, 0, w as i32, h as i32, glow::RGB, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(&mut pixel_data[0..((w * h * 3) as usize)]));
                    }
                    Framebuffer::render_to_win(ctx.screen_rect().width() as u32, ctx.screen_rect().height() as u32, gl);
                    let mut path_copy = path.clone();
                    path_copy.set_extension("");
                    path_copy.set_file_name(format!("{}-{}", path_copy.file_name().map_or("frame".to_owned(), |str| str.to_str().unwrap().to_owned()), frame));
                    path_copy.set_extension("png");
                    let _ = image::save_buffer(path_copy, pixel_data.as_slice(), w, h, image::ColorType::Rgb8);
                    *frame += 1;
                    if *frame as u32 == state.project.graphics.get(state.open_graphic).unwrap().len { 
                        self.exporting = None;
                    }
                }
                egui::Window::new("Exporting...")
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .resizable(false)
                    .auto_sized()
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.label(format!("Frame {}", frame_copy));
                    });
                ctx.request_repaint();
            }
        });
    }

}
