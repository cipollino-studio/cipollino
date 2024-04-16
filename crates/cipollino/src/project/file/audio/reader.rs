use std::{io::Read, path::PathBuf, process::{Command, Stdio}};

use crate::{audio::generate::MAX_AUDIO_CHANNELS, util::ffmpeg::FFMPEG_PATH};

pub fn read_samples(path: PathBuf, sample_rate: u32) -> Option<Vec<[f32; MAX_AUDIO_CHANNELS]>> {
    let mut process = Command::new(FFMPEG_PATH)
        .arg("-i")
        .arg(path.to_str()?)
        .arg("-filter:a")
        .arg(format!("aresample=osr={}:osf=s16:ochl=stereo", sample_rate))
        .arg("-f")
        .arg("s16le")
        .arg("pipe:1")
        .stdout(Stdio::piped())
        .spawn().ok()?;
    let mut stdout = process.stdout.take()?;
    let mut bytes = Vec::new(); 
    stdout.read_to_end(&mut bytes).ok()?;
    process.wait().ok()?;

    let mut bytes = bytes.as_slice();
    let mut result = Vec::new();
    while !bytes.is_empty() {
        let mut sample = [0.0; MAX_AUDIO_CHANNELS];
        for i in 0..MAX_AUDIO_CHANNELS {
            let value = i16::from_le_bytes([bytes[0], bytes[1]]);
            bytes = &bytes[2..];
            sample[i] = (value as f32) / (i16::MAX as f32);
        }
        result.push(sample);
    }
  
    Some(result)
}
