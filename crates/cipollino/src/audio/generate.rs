
use super::state::AudioState;

pub const MAX_AUDIO_CHANNELS: usize = 2;

impl AudioState {
    
    pub fn next_audio_sample(&mut self) -> [f32; MAX_AUDIO_CHANNELS] {
        let mut out = [0.0; MAX_AUDIO_CHANNELS];

        for clip in &self.clips {
            if self.time >= clip.begin && self.time < clip.end {
                let offset = (self.time - clip.begin + clip.offset) as usize;
                if offset >= clip.samples.len() {
                    continue;
                }
                for c in 0..MAX_AUDIO_CHANNELS {
                    out[c] += clip.samples[offset][c];
                }
            }
        }

        self.time += 1;
        out
    }

}
