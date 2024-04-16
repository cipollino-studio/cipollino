
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio::generate::MAX_AUDIO_CHANNELS;

use self::state::AudioState;

pub mod state;
pub mod generate;

pub struct AudioController {
    stream: cpal::Stream,
    pub state: Arc<Mutex<AudioState>>
}

impl AudioController {

    pub fn new() -> Result<Self, String> {
        
        let cpal_host = cpal::default_host();
        let device = cpal_host.default_output_device().ok_or("Audio output device not found.")?;
        let mut config = device.supported_output_configs().map_err(|err| err.to_string())?.next().ok_or("Audio output config missing.")?.with_max_sample_rate().config();
        config.buffer_size = cpal::BufferSize::Fixed(1000);

        let state = Arc::new(Mutex::new(AudioState::new()));
        let state_clone = state.clone(); 

        let channels = config.channels as usize;
        let stream = device.build_output_stream(&config, move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
            let state = state.clone();
            let state = &mut *state.lock().unwrap();
            for i in 0..(out.len() / channels) {
                let sample = state.next_audio_sample();
                for c in 0..channels.min(MAX_AUDIO_CHANNELS) {
                    out[i * channels + c] = sample[c];
                }
            }
        }, |_err| {
            
        }, None).unwrap();

        Ok(Self {
            stream,
            state: state_clone
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
