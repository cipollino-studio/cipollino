
use std::sync::{Arc, RwLock};

use egui_toast::{Toast, ToastKind, ToastOptions};

use crate::{audio::{state::{AudioClip, AudioState}, AudioController}, project::{action::ActionManager, graphic::Graphic, layer::{Layer, LayerKind}, obj::ObjPtr, palette::Palette, stroke::{Stroke, StrokeColor}, Project}, tools::{bucket::Bucket, color_picker::ColorPicker, line::Line, pencil::Pencil, select::Select, Tool}};

use super::{clipboard, selection::{self, Selection}};

pub struct EditorState {
    pub project: Project, 

    // Subsystems
    pub actions: ActionManager,
    pub audio: Option<AudioController>,
    pub toasts: egui_toast::Toasts,
   
    // Tools
    pub tools: Vec<Arc<RwLock<dyn Tool + Send + Sync>>>,
    pub curr_tool: Arc<RwLock<dyn Tool + Send + Sync>>,

    // Selections
    pub open_graphic: ObjPtr<Graphic>,
    pub open_palette: ObjPtr<Palette>,
    pub active_layer: ObjPtr<Layer>,
    pub selection: selection::Selection,

    // Clipboard
    pub clipboard: clipboard::Clipboard,

    // Playback
    pub time: i64, // Measured in samples
    pub playing: bool,

    // Display
    pub onion_before: i32,
    pub onion_after: i32,

    // Tool Options
    pub color: StrokeColor,
    pub stroke_r: f32,
    pub stroke_filled: bool,

    // Misc
    pub just_pasted: bool // Tracks if user pasted(Cmd+V) this frame
}

impl EditorState {

    pub fn new_with_project(project: Project) -> Self {

        let mut toasts = egui_toast::Toasts::default().anchor(egui::Align2::RIGHT_BOTTOM, egui::pos2(-10.0, -10.0));

        let audio = AudioController::new();
        if audio.is_none() {
            toasts.add(Toast {
                kind: ToastKind::Error,
                text: "Could not start audio thread. Playback will not work.".into(),
                options: ToastOptions::default().show_progress(false),
            });
        }

        let select = Arc::new(RwLock::new(Select::new()));
        let pencil = Arc::new(RwLock::new(Pencil::new()));
        let bucket = Arc::new(RwLock::new(Bucket::new()));
        let color_picker = Arc::new(RwLock::new(ColorPicker::new()));
        let line = Arc::new(RwLock::new(Line::new()));
        Self {
            project: project, 

            actions: ActionManager::new(),
            audio,
            toasts, 

            tools: vec![select.clone(), pencil, bucket, color_picker, line],
            curr_tool: select,

            open_graphic: ObjPtr::null(),
            open_palette: ObjPtr::null(),
            active_layer: ObjPtr::null(),
            selection: selection::Selection::None,

            clipboard: clipboard::Clipboard::None,

            time: 0,
            playing: false,

            onion_before: 0,
            onion_after: 0,

            color: StrokeColor::Color(glam::vec4(0.0, 0.0, 0.0, 1.0)),
            stroke_r: 5.0,
            stroke_filled: false,

            just_pasted: false
        }
    }

    pub fn new() -> Self {
        EditorState::new_with_project(Project::new())
    }

    pub fn visible_strokes(&self) -> Vec<ObjPtr<Stroke>> {
        if let Some(graphic) = self.project.graphics.get(self.open_graphic) {
            visible_strokes(&self.project, graphic, self.frame()).collect()
        } else {
            Vec::new()
        }
    }

    pub fn pause(&mut self) {
        self.selection = Selection::None;
        self.playing = false;
    }

    pub fn play(&mut self) {
        self.playing = true;
        if let Some(audio) = &mut self.audio {
            let audio_state = audio.state.clone();
            if let Some(new_state) = self.get_audio_state(self.open_graphic) {
                *audio_state.lock().unwrap() = new_state;
            }
        }
    }

    pub fn frame_rate(&self) -> f32 {
        24.0
    }

    pub fn frame_len(&self) -> f32 {
        1.0 / self.frame_rate() 
    }

    pub fn sample_rate(&self) -> f32 {
        44100.0
    }

    pub fn sample_len(&self) -> f32 {
        1.0 / self.sample_rate()
    }

    pub fn time_secs(&self) -> f32 {
        self.time as f32 * self.sample_len()
    }

    pub fn frame(&self) -> i32 {
        (self.time_secs() / self.frame_len()).floor() as i32
    }  

    pub fn delete_shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::X)
    }

    pub fn reset_tool(&mut self) {
        self.curr_tool.clone().write().unwrap().reset(self);
    }

    pub fn error_toast(&mut self, message: &str) {
        self.toasts.add(egui_toast::Toast {
            kind: egui_toast::ToastKind::Error,
            text: message.to_owned().into(),
            options: egui_toast::ToastOptions::default().show_progress(false) 
        });
    }

    pub fn get_audio_state(&self, graphic: ObjPtr<Graphic>) -> Option<AudioState> {
        let mut audio = AudioState::new();
        audio.time = self.time;

        let gfx = self.project.graphics.get(graphic)?;
        for layer in &gfx.layers {
            let layer = layer.get(&self.project);
            if layer.kind != LayerKind::Audio {
                continue;
            }
            for instance in &layer.sound_instances {
                let instance = instance.get(&self.project);
                let file = self.project.audio_files.get(&instance.audio.lookup(&self.project))?;
                audio.clips.push(AudioClip {
                    begin: instance.begin, 
                    end: instance.end, 
                    samples: file.samples.clone(),
                });
            }
        }

        Some(audio)
    }

}

pub fn visible_strokes<'a>(project: &'a Project, graphic: &'a Graphic, time: i32) -> impl Iterator<Item = ObjPtr<Stroke>> + 'a {
    graphic.layers.iter().filter(|layer| {
        let layer = layer.get(project);
        layer.show && layer.kind == LayerKind::Animation
    })
        .flat_map(move |layer| layer.get(project).get_frame_at(project, time))
        .flat_map(|frame| frame.get(project).strokes.iter())
        .map(|stroke| stroke.make_ptr())
}
