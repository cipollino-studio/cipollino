use std::path::PathBuf;

use ffmpeg::{codec, encoder, format, frame, picture, software, ChannelLayout, Dictionary, Packet, Rational};

use crate::audio::generate::MAX_AUDIO_CHANNELS;

pub struct VideoWriter {
    out_ctx: format::context::Output,
    video_encoder: encoder::Video,
    audio_encoder: encoder::Audio,
    w: u32,
    h: u32,
    fps: i32,
    pub curr_frame: i64,
    video_converter: software::scaling::Context,

    pub curr_sample: i64,
    channel_layout: ChannelLayout
}

impl VideoWriter {

    fn make_video_encoder(out_ctx: &mut format::context::Output, w: u32, h: u32, fps: i32) -> Option<encoder::Video> {
        let codec = encoder::find(codec::Id::H264)?; 
        let pixel_format = format::Pixel::YUV420P;

        let mut video_out_stream = out_ctx.add_stream().ok()?;

        let mut video_encoder = codec::Encoder::new(codec).ok()?.video().ok()?;
        video_encoder.set_width(w);
        video_encoder.set_height(h);
        video_encoder.set_aspect_ratio(Rational(1, 1));
        video_encoder.set_format(pixel_format);
        video_encoder.set_bit_rate(4096 * 1024);
        let frame_rate: Rational = Rational(fps, 1); 
        video_encoder.set_frame_rate(Some(frame_rate));
        video_encoder.set_time_base(Some(frame_rate.invert()));
        video_encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        let mut opts = Dictionary::new();
        opts.set("preset", "veryslow"); 
        let video_encoder = video_encoder.open_with(opts).ok()?;

        video_out_stream.set_parameters(video_encoder.parameters());
        video_out_stream.set_time_base(Some(frame_rate.invert()));

        Some(video_encoder)
    }

    fn make_audio_encoder(out_ctx: &mut format::context::Output, sample_rate: u32) -> Option<(encoder::Audio, ChannelLayout)> {
        let codec = encoder::find(codec::Id::AAC)?;
        
        let mut audio_out_stream = out_ctx.add_stream().ok()?;
        let mut audio_encoder = codec::Encoder::new(codec).ok()?.audio().ok()?;
        audio_encoder.set_sample_rate(sample_rate);
        audio_encoder.set_channels(MAX_AUDIO_CHANNELS as i32);
        audio_encoder.set_format(format::Sample::F32(format::sample::Type::Planar));
        let channel_layout = ChannelLayout::default(MAX_AUDIO_CHANNELS as i32);
        audio_encoder.set_channel_layout(channel_layout);
        let time_base = Rational(1, sample_rate as i32);
        audio_encoder.set_time_base(Some(time_base));
        let audio_encoder = audio_encoder.open().ok()?;

        audio_out_stream.set_parameters(audio_encoder.parameters());
        audio_out_stream.set_time_base(audio_encoder.time_base());

        Some((audio_encoder, channel_layout))
    }

    pub fn new(out: PathBuf, w: u32, h: u32, fps: i32, sample_rate: u32) -> Option<Self> {

        let mut out_ctx = format::output(&out).ok()?;
        let video_encoder = Self::make_video_encoder(&mut out_ctx, w, h, fps)?;
        let (audio_encoder, channel_layout) = Self::make_audio_encoder(&mut out_ctx, sample_rate)?;
        
        format::context::output::dump(&out_ctx, 0, None);
        out_ctx.write_header().unwrap();

        Some(Self {
            out_ctx,
            video_encoder,
            audio_encoder,
            w,
            h,
            fps,
            curr_frame: 0,
            video_converter: software::converter((w, h), format::Pixel::RGB24, format::Pixel::YUV420P).ok()?,
            curr_sample: 0,
            channel_layout
        })

    }

    pub fn write_frame(&mut self, data: Vec<u8>) {

        let timestamp = self.curr_frame;

        let mut frame_rgb = frame::Video::new(format::Pixel::RGB24, self.w, self.h);
        let plane = frame_rgb.plane_mut::<(u8, u8, u8)>(0);
        for i in 0..plane.len() {
            plane[i] = (data[3 * i], data[3 * i + 1], data[3 * i + 2]);
        }

        let mut frame = frame::Video::empty();
        self.video_converter.run(&frame_rgb, &mut frame).unwrap();

        frame.set_pts(Some(timestamp));
        frame.set_kind(picture::Type::None);
        
        let _ = self.video_encoder.send_frame(&frame);

        self.process_encoded_video_packets();

        self.curr_frame += 1;
    }

    pub fn write_sound(&mut self, data: Vec<[f32; MAX_AUDIO_CHANNELS]>) {
        let mut audio_frame = frame::Audio::new(format::Sample::F32(format::sample::Type::Planar), 1024, self.channel_layout);
        audio_frame.set_pts(Some(self.curr_sample));
        let left_plane = audio_frame.plane_mut::<f32>(0);
        for i in 0..data.len() {
            left_plane[i] = data[i][0];
        }
        let right_plane = audio_frame.plane_mut::<f32>(1);
        for i in 0..data.len() {
            right_plane[i] = data[i][1];
        }

        self.audio_encoder.send_frame(&audio_frame).unwrap();

        self.process_encoded_audio_packets();

        self.curr_sample += data.len() as i64;
    }

    pub fn close(&mut self) {

        self.video_encoder.send_eof().unwrap();
        self.audio_encoder.send_eof().unwrap();
        self.process_encoded_video_packets();
        self.process_encoded_audio_packets();

        self.out_ctx.write_trailer().unwrap();

    }

    fn process_encoded_video_packets(&mut self) {
        let mut packet = Packet::empty();
        while self.video_encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.rescale_ts(Rational(1, self.fps), Rational(1, ((self.fps as f32) * 517.172).floor() as i32));
            packet.write_interleaved(&mut self.out_ctx).unwrap();
        }
    }

    fn process_encoded_audio_packets(&mut self) {
        let mut packet = Packet::empty();
        while self.audio_encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(1);
            packet.write_interleaved(&mut self.out_ctx).unwrap();
        }
    }

}
