
use crate::{editor::state::EditorState, project::{layer::{Layer, LayerKind}, obj::ObjBox, sound_instance::SoundInstance}};

pub const MAX_AUDIO_CHANNELS: usize = 2;

impl EditorState {

    pub fn next_audio_sample(&mut self) -> [f32; MAX_AUDIO_CHANNELS] {
        if !self.playing {
            return [0.0, 0.0];
        }

        let mut out = [0.0, 0.0];
        
        if let Some(gfx) = self.project.graphics.get(self.open_graphic) {
            for layer in &gfx.layers {
                self.add_layer_audio(layer, &mut out);    
            }
        }

        self.time += 1;

        out
    }

    fn add_layer_audio(&self, layer: &ObjBox<Layer>, out: &mut [f32; MAX_AUDIO_CHANNELS]) {
        let layer = layer.get(&self.project);
        if !layer.show {
            return;
        }

        if layer.kind == LayerKind::Audio {
            for sound_instance in &layer.sound_instances {
                self.add_sound_instance_audio(sound_instance, out);
            }
        }
    }

    fn add_sound_instance_audio(&self, sound_instance: &ObjBox<SoundInstance>, out: &mut [f32; MAX_AUDIO_CHANNELS]) {
        let sound_instance = sound_instance.get(&self.project);
        if self.time < sound_instance.begin || self.time >= sound_instance.end {
            return; 
        }

        if let Some(audio) = self.project.audio_files.get(&sound_instance.audio.lookup(&self.project)) {
            for c in 0..MAX_AUDIO_CHANNELS {
                let sample_idx = self.time - sound_instance.begin; 
                let sample = sample_idx as usize;
                if sample < audio.samples.len() {
                    out[c] += audio.samples[sample][c];
                }
            }
        }
    }
    
}
