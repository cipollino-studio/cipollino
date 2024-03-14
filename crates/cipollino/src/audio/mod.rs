
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{audio::generate::MAX_AUDIO_CHANNELS, editor::state::EditorState};

pub mod generate;

pub struct AudioController {
    stream: cpal::Stream,
}

impl AudioController {

    pub fn new(state: Arc<Mutex<EditorState>>) -> Option<Self> {
        
        let cpal_host = cpal::default_host();
        let device = cpal_host.default_output_device()?;
        let mut config = device.supported_output_configs().ok()?.next()?.with_max_sample_rate().config();
        config.buffer_size = cpal::BufferSize::Fixed(1000);

        let channels = config.channels as usize;
        let stream = device.build_output_stream(&config, move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for i in 0..(out.len() / channels) {
                let state = &mut *state.lock().unwrap();
                let sample = state.next_audio_sample();
                for j in 0..channels.min(MAX_AUDIO_CHANNELS) {
                    out[i * channels + j] = sample[j]; 
                }
            }
        }, |_err| {
            
        }, None).unwrap();

        Some(Self {
            stream,
        })
    }

    pub fn set_playing(&mut self, play: bool) {
        if play {
            let _ = self.stream.play();
        } else {
            let _ = self.stream.pause();
        }
    }

}
