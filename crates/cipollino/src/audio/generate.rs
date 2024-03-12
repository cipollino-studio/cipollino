
use crate::{editor::EditorState, project::layer::LayerKind};

pub const MAX_AUDIO_CHANNELS: usize = 2;

impl EditorState {

    pub fn next_audio_sample(&mut self) -> [f32; MAX_AUDIO_CHANNELS] {
        if !self.playing {
            return [0.0, 0.0];
        }
        self.time += 1;

        let mut out = [0.0, 0.0];
        
        if let Some(gfx) = self.project.graphics.get(self.open_graphic) {

            for layer in &gfx.layers {
                let layer = layer.get(&self.project);
                if !layer.show {
                    continue;
                }

                if layer.kind == LayerKind::Audio {
                    for sound_instance in &layer.sound_instances {
                        let sound_instance = sound_instance.get(&self.project);
                        if self.time >= sound_instance.begin && self.time < sound_instance.end {
                            if let Some(audio) = self.project.audio_files.get(&sound_instance.audio.lookup(&self.project)) {
                                out[0] += audio.samples[(self.time as usize - sound_instance.begin as usize) % audio.samples.len()][0];
                            }
                        }
                    }
                }
            }
        }

        out
    }
    
}
