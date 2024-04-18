
use std::{path::PathBuf, sync::{Arc, RwLock}};

use crate::{project::{action::ActionManager, graphic::Graphic, layer::{Layer, LayerKind}, obj::ObjPtr, palette::Palette, stroke::{Stroke, StrokeColor}, Project}, tools::{bucket::Bucket, color_picker::ColorPicker, line::Line, pencil::Pencil, select::Select, Tool}};

use super::{clipboard, selection::{self, Selection}, toasts::Toasts};

pub struct EditorState {
    pub project: Project, 

    // Subsystems
    pub actions: ActionManager,
   
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

        let select = Arc::new(RwLock::new(Select::new()));
        let pencil = Arc::new(RwLock::new(Pencil::new()));
        let bucket = Arc::new(RwLock::new(Bucket::new()));
        let color_picker = Arc::new(RwLock::new(ColorPicker::new()));
        let line = Arc::new(RwLock::new(Line::new()));
        Self {
            project: project, 

            actions: ActionManager::new(),
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
        EditorState::new_with_project(Project::new("".into(), 24.0, 44100.0))
    }

    pub fn load_project(path: PathBuf, toasts: &mut Toasts) -> Self {
        let (project, metadata) = Project::load(path);
        let mut state = Self::new_with_project(project);
        metadata.display_errors(&mut state, toasts);

        state
    }

    fn visible_strokes_in_layer(&self, layer: &Layer, time: i32, strokes: &mut Vec<ObjPtr<Stroke>>) {
        if !layer.show {
            return;
        }
        if layer.kind == LayerKind::Animation {
            if let Some(frame) = layer.get_frame_at(&self.project, time) {
                let frame = frame.get(&self.project);
                for stroke in &frame.strokes {
                    strokes.push(stroke.make_ptr());
                }
            }
        } else if layer.kind == LayerKind::Group {
            for layer in &layer.layers {
                self.visible_strokes_in_layer(layer.get(&self.project), time, strokes);
            } 
        }
    }

    pub fn visible_strokes(&self) -> Vec<ObjPtr<Stroke>> {
        let mut res = Vec::new();
        if let Some(graphic) = self.project.graphics.get(self.open_graphic) {
            for layer in &graphic.layers {
                self.visible_strokes_in_layer(layer.get(&self.project), self.frame(), &mut res)
            }            
        }
        res
    }

    pub fn pause(&mut self) {
        self.selection = Selection::None;
        self.playing = false;
    }

    pub fn play(&mut self) {
        self.playing = true;
        
    }

    pub fn frame_rate(&self) -> f32 {
        self.project.fps
    }

    pub fn frame_len(&self) -> f32 {
        1.0 / self.frame_rate() 
    }

    pub fn sample_rate(&self) -> f32 {
        self.project.sample_rate
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

    pub fn reset_tool(&mut self) {
        self.curr_tool.clone().write().unwrap().reset(self);
    }

}
