
use std::sync::Arc;

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
