
use std::{io::Write, path::PathBuf, process::{Command, Stdio}, thread::{self, JoinHandle}};

use unique_type_id::UniqueTypeId;

use crate::{audio::{generate::MAX_AUDIO_CHANNELS, state::AudioState}, editor::{dialog::Dialog, state::EditorState, EditorSystems}, project::{graphic::Graphic, obj::{obj_list::ObjListTrait, ObjPtr}}, renderer::fb::Framebuffer, util::ffmpeg::FFMPEG_PATH};

use super::video_writer::VideoWriter;

use glow::HasContext;

enum ExportState {
    Audio {
        thread: JoinHandle<()>,
        audio_path: PathBuf
    },
    Video {
        writer: VideoWriter,
        fb: Framebuffer,
        aa_fb: Framebuffer,
        curr_frame: i32,
        audio_path: PathBuf
    }
}

#[derive(UniqueTypeId)]
pub struct ExportProgressDialog {
    gfx: ObjPtr<Graphic>,
    out_path: PathBuf,
    state: ExportState
}

fn audio_encoding_thread(out_path: PathBuf, mut audio_state: AudioState, len: i64) {
    let mut process = Command::new(FFMPEG_PATH)
        .arg("-y") // Override output
        .arg("-f") // Input format
        .arg("s16le")
        .arg("-ac")
        .arg(format!("{}", MAX_AUDIO_CHANNELS))
        .arg("-i")
        .arg("-")
        .arg(out_path.to_str().unwrap())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();
    let mut stdin = process.stdin.take().unwrap();
    let _stderr = process.stderr.take().unwrap();
    let _stdout = process.stdout.take().unwrap();
    let mut byte_buffer = Vec::new();

    while audio_state.time < len {
        let sample = audio_state.next_audio_sample(); 
        for i in 0..MAX_AUDIO_CHANNELS {
            let sample = sample[i];
            let sample = (sample * (i16::MAX as f32)) as i16;
            byte_buffer.extend_from_slice(&sample.to_le_bytes());
        }
    }
    let _ = stdin.write_all(&byte_buffer);

    drop(stdin);

    process.wait().unwrap();
}

impl ExportProgressDialog {

    pub fn new(out_path: PathBuf, gfx_ptr: ObjPtr<Graphic>, state: &EditorState, _systems: &EditorSystems) -> Result<Self, String> {
        let gfx = state.project.graphics.get(gfx_ptr).ok_or("Graphic missing")?;

        let mut audio_state = state.get_audio_state(gfx_ptr).ok_or("Could not initialize audio state.")?;
        audio_state.time = 0;
        let gfx_len_in_samples = ((gfx.len as f32) * state.frame_len() * state.sample_rate()) as i64; 
        let audio_file_path = "test.wav";
        let audio_export_thread = thread::spawn(move || {
            audio_encoding_thread(audio_file_path.into(), audio_state, gfx_len_in_samples); 
        });

        Ok(Self {
            gfx: gfx_ptr,
            out_path: out_path,
            state: ExportState::Audio {
                thread: audio_export_thread,
                audio_path: "test.wav".into()
            } 
        })
    }

}

impl Dialog for ExportProgressDialog {

    fn render(&mut self, ui: &mut egui::Ui, state: &mut EditorState, systems: &mut EditorSystems) -> bool {
        let message = match self.state {
            ExportState::Audio { .. } => "Generating audio",
            ExportState::Video { .. } => "Encoding video",
        };

        let n_dots = ui.ctx().input(|i| i.time).floor() as usize % 3 + 1;
        ui.label(message.to_owned() + &".".repeat(n_dots));
        ui.ctx().request_repaint();

        match &mut self.state {
            ExportState::Audio { thread, audio_path } => {
                let gfx = if let Some(gfx) = state.project.graphics.get(self.gfx) {
                    gfx
                } else {
                    systems.toasts.error_toast("Export failed: Graphic missing.");
                    return true;
                };

                let writer = match VideoWriter::new(self.out_path.clone(), audio_path.clone(), gfx.w, gfx.h, state.project.fps as i32) {
                    Ok(writer) => writer,
                    Err(err) => {
                        systems.toasts.error_toast(format!("Export failed: {}", err));
                        return true;
                    },
                };

                if thread.is_finished() {
                    self.state = ExportState::Video {
                        writer,
                        fb: Framebuffer::new(gfx.w, gfx.h, systems.gl),
                        aa_fb: Framebuffer::new(gfx.w * 2, gfx.h * 2, systems.gl),
                        curr_frame: 0,
                        audio_path: audio_path.clone()
                    };
                }
            },
            ExportState::Video{ writer, fb, aa_fb, curr_frame, audio_path } => {
                let gfx = if let Some(gfx) = state.project.graphics.get(self.gfx) {
                    gfx
                } else {
                    systems.toasts.error_toast("Export failed: Graphic missing.");
                    return true;
                };
                
                if *curr_frame == gfx.len as i32 {
                    if writer.done() {
                        let _ = std::fs::remove_file(audio_path);
                        return true;
                    } else {
                        return false;
                    }
                } 

                let aa_scl = 2;
                let w = gfx.w;
                let h = gfx.h;
                let gfx_len = gfx.len as i32;

                systems.renderer.render(fb, None, w * aa_scl, h * aa_scl, glam::Vec2::ZERO, h as f32 / 2.0, &mut state.project, self.gfx, *curr_frame, 0, 0, systems.gl);
                aa_fb.resize(w, h, systems.gl);
                unsafe {
                    systems.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(aa_fb.fbo));
                    systems.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(fb.fbo));
                    systems.gl.blit_framebuffer(0, (h * aa_scl - 1) as i32, (w * aa_scl) as i32, 0, 0, 0, w as i32, h as i32, glow::COLOR_BUFFER_BIT, glow::LINEAR);
                }

                let mut pixel_data = vec![0; (w * h * 3) as usize]; 

                unsafe {
                    systems.gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(aa_fb.fbo));
                    systems.gl.read_pixels(0, 0, w as i32, h as i32, glow::RGB, glow::UNSIGNED_BYTE, glow::PixelPackData::Slice(&mut pixel_data[0..((w * h * 3) as usize)]));
                }

                Framebuffer::render_to_win(ui.ctx().screen_rect().width() as u32, ui.ctx().screen_rect().height() as u32, systems.gl);

                if let Err(msg) = writer.write_frame(pixel_data) {
                    systems.toasts.error_toast(format!("Export failed: {}", msg));
                    return true;
                }
                *curr_frame += 1;

                if *curr_frame == gfx_len {
                    let _ = writer.close();
                }
            },
        }

        false
    }

    fn title(&self, _state: &EditorState) -> String {
        "".to_owned()
    }

    fn unique_dialog() -> bool {
        true 
    }

    fn anchor(&self) -> Option<egui::Align2> {
        Some(egui::Align2::CENTER_CENTER)
    }

    fn resizable(&self) -> bool {
        false
    }

    fn show_title(&self) -> bool {
        false
    }

    fn margin(&self) -> Option<egui::Margin> {
        Some(egui::Margin::same(20.0))
    }

}
