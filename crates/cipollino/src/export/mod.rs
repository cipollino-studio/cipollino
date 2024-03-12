
use std::path::PathBuf;

use glow::HasContext;

use crate::{audio::generate::MAX_AUDIO_CHANNELS, editor::{EditorRenderer, EditorState}, project::{graphic::Graphic, obj::ObjPtr}, renderer::fb::Framebuffer};

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
                        let mut audio_path = path.clone();
                        self.exporting = Some((path, 0, state.open_graphic));
                        close_dialog = true;

                        // TODO: really scuffed, redo the entire export system honestly
                        let mut audio = Vec::new();
                        state.time = 0;
                        state.playing = true;
                        
                        let gfx_len = (state.project.graphics.get(state.open_graphic).unwrap().len as f32 * state.frame_len() / state.sample_len()) as usize;
                        for _i in 0..gfx_len {
                            let [left, right] = state.next_audio_sample();
                            audio.push((left * (i16::MAX as f32)) as i16);
                            audio.push((right * (i16::MAX as f32)) as i16);
                        }

                        audio_path.set_extension("wav");
                        if let Ok(mut audio_file) = std::fs::File::create(audio_path) {
                            wav::write(wav::Header::new(1, MAX_AUDIO_CHANNELS as u16, state.sample_rate() as u32, 16), &wav::BitDepth::Sixteen(audio), &mut audio_file);
                        }

                        state.playing = false;
                    }
                }
            });
        if close_dialog {
            self.dialog_open = false;
        }
        
        if let Some((path, frame, gfx_key)) = &mut self.exporting {
            let frame_copy = *frame;
            if let None = self.fb {
                self.fb = Some((
                    Framebuffer::new(1920, 1080, renderer.gl),
                    Framebuffer::new(1920, 1080, renderer.gl)
                ));
            } 
            if let Some((fb, aa_fb)) = self.fb.as_mut() {
                let aa_scl = 2;
                let gfx = state.project.graphics.get(*gfx_key).unwrap();
                let w = gfx.w;
                let h = gfx.h;
                renderer.renderer.render(fb, None, w * aa_scl, h * aa_scl, glam::Vec2::ZERO, h as f32 / 2.0, &mut state.project, *gfx_key, *frame, 0, 0, renderer.gl);
                aa_fb.resize(w, h, renderer.gl);
                unsafe {
                    renderer.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(aa_fb.fbo));
                    renderer.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fb.fbo));
                    renderer.gl.blit_framebuffer(0, (h * aa_scl - 1) as i32, (w * aa_scl) as i32, 0, 0, 0, w as i32, h as i32, glow::COLOR_BUFFER_BIT, glow::LINEAR);
                }
                let mut pixel_data = Vec::new();
                // TODO: this is probably inneficient
                pixel_data.reserve((w * h * 3) as usize);
                for _i in 0..(w * h * 3) {
                    pixel_data.push(0);
                }
                unsafe {
                    renderer.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(aa_fb.fbo));
                    renderer.gl.read_pixels(0, 0, w as i32, h as i32, glow::RGB, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(&mut pixel_data[0..((w * h * 3) as usize)]));
                }
                Framebuffer::render_to_win(ctx.screen_rect().width() as u32, ctx.screen_rect().height() as u32, renderer.gl);
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
    }

}
