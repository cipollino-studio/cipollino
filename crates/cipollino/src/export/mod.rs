
use std::{path::PathBuf, sync::{mpsc, Arc, Mutex}, thread::JoinHandle, time::SystemTime};

mod video_writer;

use egui::ProgressBar;
use glow::HasContext;

use crate::{audio::generate::MAX_AUDIO_CHANNELS, editor::{state::EditorState, EditorSystems}, project::{graphic::Graphic, obj::ObjPtr}, renderer::fb::Framebuffer, util::ui::path::path_selector};

use self::video_writer::VideoWriter;

enum ExportThreadMessage {
    Frame(Vec<u8>),
    Sound(Vec<[f32; MAX_AUDIO_CHANNELS]>),
    StopVideo,
    StopAudio
}

enum ExportStatus {
    NotExporting,
    ExportingFrames {
        graphic: ObjPtr<Graphic>,
        thread_tx: mpsc::Sender<ExportThreadMessage>,
        curr_frame: i32,
        frames: i32,

        progress: Arc<Mutex<(i64, i64)>>,
        encoding_thread: JoinHandle<()>
    }
}

pub struct Export {
    fb: Option<(Framebuffer, Framebuffer)>,
    pub dialog_open: bool,
    pub path: PathBuf,
    status: ExportStatus
}

impl Export {
    
    pub fn new() -> Self {
        Self {
            fb: None,
            dialog_open: false,
            path: PathBuf::new(),
            status: ExportStatus::NotExporting 
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, state: &mut EditorState, systems: &mut EditorSystems) {
        self.export_dialog(ctx, state);
        self.render_frames(ctx, state, systems);
        self.render_progress(ctx, state);

        if let ExportStatus::ExportingFrames { encoding_thread, .. } = &self.status {
            if encoding_thread.is_finished() {
                self.status = ExportStatus::NotExporting;
            }
        }
    }

    fn export_dialog(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let mut dialog_open = self.dialog_open;
        let mut close_dialog = false;
        egui::Window::new("Export")
            .open(&mut dialog_open)
            .show(ctx, |ui| {
                let open_graphic = state.project.graphics.get(state.open_graphic);
                if open_graphic.is_none() {
                    ui.centered_and_justified(|ui| {
                        ui.label("Open a clip to export it.");
                    });
                    return;
                }
                let open_graphic = open_graphic.unwrap();
                if !open_graphic.clip {
                    ui.centered_and_justified(|ui| {
                        ui.label("Cannot export non-clip.");
                    });
                    return;
                }
                path_selector(ui, &mut self.path, false, |path| {
                    path.set_extension("mp4");
                });
                if ui.button("Export").clicked() {
                    self.begin_export(self.path.clone(), state, state.open_graphic);
                    close_dialog = true;
                }
            });
        if close_dialog {
            dialog_open = false;
        }
        self.dialog_open = dialog_open;
    }

    fn begin_export(&mut self, mut export_path: PathBuf, state: &mut EditorState, graphic_ptr: ObjPtr<Graphic>) {
        let graphic = if let Some(graphic) = state.project.graphics.get(graphic_ptr) {
            graphic
        } else {
            return;
        };
        let w = graphic.w;
        let h = graphic.h;
        let fps = state.frame_rate() as i32;
        let sample_rate = state.sample_rate() as u32;

        export_path.set_extension("mp4");

        let (tx, rx) = mpsc::channel();
        
        let progress = Arc::new(Mutex::new((0, 0)));
        let progress_copy = progress.clone();

        // Encoder thread
        let encoding_thread = std::thread::spawn(move || {
            let mut writer = if let Some(writer) = VideoWriter::new(export_path, w, h, fps, sample_rate) {
                writer
            } else {
                return;
            };

            let mut video_stopped = false;
            let mut audio_stopped = false;

            loop {
                let msg = if let Ok(msg) = rx.recv() {
                    msg
                } else {
                    break;
                };

                match msg {
                    ExportThreadMessage::Frame(data) => {
                        writer.write_frame(data);
                    },
                    ExportThreadMessage::Sound(data) => {
                        writer.write_sound(data);
                    },
                    ExportThreadMessage::StopVideo => {
                        video_stopped = true;
                    },
                    ExportThreadMessage::StopAudio => {
                        audio_stopped = true;
                    }
                }

                *progress.lock().unwrap() = (writer.curr_frame, writer.curr_sample);

                if video_stopped && audio_stopped {
                    break;
                }

            }

            writer.close();
        });

        // Audio generation thread
        let audio_tx = tx.clone();
        let mut audio_state = state.get_audio_state(graphic_ptr).unwrap();
        audio_state.time = 0;
        let gfx_len_in_samples = ((graphic.len as f32) * state.frame_len() * state.sample_rate()) as i64; 
        std::thread::spawn(move || {
            while audio_state.time < gfx_len_in_samples {
                let block_size = (gfx_len_in_samples - audio_state.time).min(1024) as usize;
                let mut data = Vec::new();
                data.reserve(block_size);
                for _ in 0..block_size {
                    data.push(audio_state.next_audio_sample());
                }
                let _ = audio_tx.send(ExportThreadMessage::Sound(data));
            }
            let _ = audio_tx.send(ExportThreadMessage::StopAudio);
        });

        self.status = ExportStatus::ExportingFrames {
            thread_tx: tx,
            graphic: graphic_ptr,
            curr_frame: 0,
            frames: graphic.len as i32,

            progress: progress_copy,
            encoding_thread
        };
    }

    fn render_frames(&mut self, ctx: &egui::Context, state: &mut EditorState, systems: &mut EditorSystems) {

        match &self.status {
            ExportStatus::NotExporting => return,
            ExportStatus::ExportingFrames { curr_frame, frames, .. } => {
                if curr_frame == frames {
                    return;
                }

                let time = SystemTime::now();
                while time.elapsed().map(|elapsed| elapsed.as_secs_f32() < 0.05).unwrap_or(false) {
                    self.render_frame(ctx, state, systems);
                }
            }
        } 

    }

    fn render_frame(&mut self, ctx: &egui::Context, state: &mut EditorState, systems: &mut EditorSystems) {
        if let ExportStatus::ExportingFrames {
            graphic,
            thread_tx,
            curr_frame,
            frames,
            .. } = &mut self.status {

                if *curr_frame == *frames {
                    return;
                }

                if let None = self.fb {
                    self.fb = Some((
                        Framebuffer::new(1920, 1080, systems.gl),
                        Framebuffer::new(1920, 1080, systems.gl)
                    ));
                } 
                if let Some((fb, aa_fb)) = self.fb.as_mut() {
                    let aa_scl = 2;
                    let gfx = state.project.graphics.get(*graphic).unwrap();
                    let w = gfx.w;
                    let h = gfx.h;
                    let gfx_len = gfx.len;
                    systems.renderer.render(fb, None, w * aa_scl, h * aa_scl, glam::Vec2::ZERO, h as f32 / 2.0, &mut state.project, *graphic, *curr_frame, 0, 0, systems.gl);
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

                    Framebuffer::render_to_win(ctx.screen_rect().width() as u32, ctx.screen_rect().height() as u32, systems.gl);

                    let _ = thread_tx.send(ExportThreadMessage::Frame(pixel_data));

                    *curr_frame += 1;

                    if *curr_frame == gfx_len as i32 {
                        let _ = thread_tx.send(ExportThreadMessage::StopVideo);
                    } 
                }
                ctx.request_repaint();
        }
    }

    fn render_progress(&mut self, ctx: &egui::Context, state: &EditorState) {
        if let ExportStatus::ExportingFrames {
            frames,
            progress,
            .. } = &self.status {
                egui::Window::new("Export Progress").show(ctx, |ui| {
                    let (curr_frame, curr_sample) = *progress.lock().unwrap(); 

                    ui.add(ProgressBar::new((curr_frame as f32) / (*frames as f32)).text("Video"));

                    let length_in_samples = (*frames as f32) * state.frame_len() * state.sample_rate();
                    ui.add(ProgressBar::new((curr_sample as f32) / length_in_samples).text("Audio"));
                });
        }
    }

    pub fn exporting(&self) -> bool {
        match self.status {
            ExportStatus::NotExporting => false,
            ExportStatus::ExportingFrames { .. } => true,
        }
    }

}
