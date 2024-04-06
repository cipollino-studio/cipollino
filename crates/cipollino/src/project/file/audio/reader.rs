use std::path::PathBuf;

use ffmpeg::{decoder, format, frame, media, software::{resampler, resampling}};

use crate::audio::generate::MAX_AUDIO_CHANNELS;

fn process_decoded_packets(decoder: &mut decoder::Audio, audio_converter: &mut resampling::Context, samples: &mut Vec<[f32; MAX_AUDIO_CHANNELS]>) -> Option<()> {
    let mut decoded_frame = frame::Audio::empty(); 
    let mut converted_frame = frame::Audio::empty();  
    while decoder.receive_frame(&mut decoded_frame).is_ok() {
        audio_converter.run(&decoded_frame, &mut converted_frame).ok()?;
        let frame_length = converted_frame.plane::<i16>(0).len();
        for i in 0..frame_length {
            let mut sample = [0.0; MAX_AUDIO_CHANNELS];
            for j in 0..MAX_AUDIO_CHANNELS {
                sample[j] = (converted_frame.plane::<i16>(j)[i] as f32) / (i16::MAX as f32);
            }
            samples.push(sample);
        }
    }

    Some(())
}

pub fn read_samples(path: PathBuf, sample_rate: u32) -> Option<Vec<[f32; MAX_AUDIO_CHANNELS]>> {
    let mut input_ctx = format::input(path).ok()?;
    let audio_input = input_ctx.streams().best(media::Type::Audio)?;
    let mut decoder = audio_input.decoder().ok()?.audio().ok()?;
    decoder.set_parameters(audio_input.parameters()).ok()?;

    let mut audio_converter = resampler(
        (decoder.format(), decoder.channel_layout(), decoder.sample_rate()),
        (format::Sample::I16(format::sample::Type::Planar), ffmpeg::ChannelLayout::default(MAX_AUDIO_CHANNELS as i32), sample_rate)
    ).ok()?; 

    let mut samples = Vec::new();

    for packet in input_ctx.packets() {
        let (_, packet) = packet.ok()?;
        decoder.send_packet(&packet).ok()?;
        process_decoded_packets(&mut decoder, &mut audio_converter, &mut samples);
    }

    decoder.send_eof().ok()?;
    process_decoded_packets(&mut decoder, &mut audio_converter, &mut samples);

    Some(samples)
}
