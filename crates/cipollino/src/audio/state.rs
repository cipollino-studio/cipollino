
use std::sync::Arc;

use crate::{editor::state::EditorState, project::{graphic::Graphic, layer::{Layer, LayerKind}, obj::{obj_list::ObjListTrait, ObjBox, ObjPtr}}};

use super::generate::MAX_AUDIO_CHANNELS;

pub struct AudioClip {
    pub begin: i64,
    pub end: i64,
    pub offset: i64,
    pub samples: Arc<Vec<[f32; MAX_AUDIO_CHANNELS]>>
}

pub struct AudioState {
    pub clips: Vec<AudioClip>,
    pub time: i64
}

impl AudioState {

    pub fn new() -> Self {
        Self {
            clips: Vec::new(),
            time: 0
        }
    }

}

impl EditorState {

    fn add_layers_to_audio_state(&self, layers: &Vec<ObjBox<Layer>>, audio: &mut AudioState) {
        for layer in layers {
            let layer = layer.get(&self.project);
            if !layer.show {
                continue; // Layer muted
            }
            if layer.kind == LayerKind::Audio {
                for instance in &layer.sound_instances {
                    let instance = instance.get(&self.project);
                    if let Some(file) = self.project.audio_files.get(&instance.audio) {
                        audio.clips.push(AudioClip {
                            begin: instance.begin, 
                            end: instance.end, 
                            offset: instance.offset,
                            samples: file.data.samples.clone(),
                        });
                    }
                }
            } else if layer.kind == LayerKind::Group {
                self.add_layers_to_audio_state(&layer.layers, audio);
            }
        }
    }

    pub fn get_audio_state(&self, graphic: ObjPtr<Graphic>) -> Option<AudioState> {
        let mut audio = AudioState::new();
        audio.time = self.time;
        let gfx = self.project.graphics.get(graphic)?;
        self.add_layers_to_audio_state(&gfx.layers, &mut audio);
        Some(audio)
    }
    
}