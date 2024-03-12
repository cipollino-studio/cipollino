
use std::{fs::File, path::PathBuf};

use crate::audio::generate::MAX_AUDIO_CHANNELS;

use super::FileType;

pub const SAMPLES_PER_VOLUME_SUM: usize = 100;

#[derive(Clone)]
pub struct AudioFile {
    pub samples: Vec<[f32; MAX_AUDIO_CHANNELS]>,
    pub volumes: Vec<f32>
}

impl FileType for AudioFile {

    fn list_in_folder(folder: &crate::project::folder::Folder) -> &Vec<super::FilePtr<Self>> {
        &folder.audios
    }

    fn list_in_folder_mut(folder: &mut crate::project::folder::Folder) -> &mut Vec<super::FilePtr<Self>> {
        &mut folder.audios
    }

}

fn read_mp3_data(file: File) -> Vec<[f32; MAX_AUDIO_CHANNELS]> {
    use minimp3_fixed::{Decoder, Frame, Error}; 
    
    let mut decoder = Decoder::new(file);
    let mut samples = Vec::new();
    loop {
        match decoder.next_frame() {
            Ok(Frame { data, sample_rate: _, channels, .. }) => {
                // TODO: reinteroplate audio with sample rates not equal to project's sample rate 
                for i in 0..(data.len() / channels) {
                    let mut sample = [0.0; MAX_AUDIO_CHANNELS];
                    for j in 0..(channels.min(MAX_AUDIO_CHANNELS)) {
                        sample[j] = (data[i * channels + j] as f32) / (i16::MAX as f32);
                    }

                    // If the source audio is mono, copy the one channel to all channels 
                    if channels == 1 {
                        for i in 1..MAX_AUDIO_CHANNELS {
                            sample[i] = sample[0];
                        }
                    }

                    samples.push(sample);
                }
            },
            Err(Error::Eof) => break,
            Err(_) => break
        }    
    }

    samples
}

impl AudioFile {

    pub fn new(samples: Vec<[f32; 2]>) -> Self {
        let mut volumes = Vec::new();
        for i in 0..samples.len() / SAMPLES_PER_VOLUME_SUM {
            let mut volume = 0.0;
            for j in 0..SAMPLES_PER_VOLUME_SUM {
                for c in 0..MAX_AUDIO_CHANNELS {
                    volume += samples[i * SAMPLES_PER_VOLUME_SUM + j][c].abs();
                }
            }
            volume /= (MAX_AUDIO_CHANNELS * SAMPLES_PER_VOLUME_SUM) as f32;
            volumes.push(volume);
        }

        Self {
            samples,
            volumes
        }
    }

    pub fn load(path: &PathBuf) -> Option<Self> {
        match path.extension()?.to_str()? {
            "mp3" => {
                let file = File::open(path).ok()?;
                let samples = read_mp3_data(file); 
                Some(Self::new(samples))
            },
            _ => None
        }
    }

}
