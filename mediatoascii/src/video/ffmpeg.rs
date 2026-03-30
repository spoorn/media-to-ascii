use ffmpeg_next::codec::Id;
use ffmpeg_next::format::{Pixel, input, output};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video as FfmpegVideoFrame;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::image::generate_ascii_image;
use crate::util::FFmpegFrame;
use crate::util::constants::{GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE};
use crate::video::VideoConfig;
use crate::video::VideoResult;
use crate::video::errors::Error;

pub struct FFmpegVideoReader {
    pub context: ffmpeg_next::format::context::Input,
    pub video_stream_index: usize,
    pub total_frames: u64,
    pub fps: f64,
    pub decoder: ffmpeg_next::codec::decoder::video::Video,
    pub scaler: Context,
}

impl FFmpegVideoReader {
    pub fn new(path: &str) -> VideoResult<Self> {
        ffmpeg_next::init().map_err(|e| Error::VideoReadError(format!("ffmpeg init error: {e}")))?;

        let context = input(path).map_err(|e| Error::VideoReadError(format!("ffmpeg input error: {e}")))?;

        let video_stream = context
            .streams()
            .best(Type::Video)
            .ok_or_else(|| Error::VideoReadError("ffmpeg error: no video stream found".to_string()))?;
        let video_stream_index = video_stream.index();

        let total_frames = video_stream.frames() as u64;

        let video_fps = video_stream.avg_frame_rate();
        let fps = if video_fps.denominator() != 0 {
            video_fps.numerator() as f64 / video_fps.denominator() as f64
        } else {
            0.0
        };

        let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
            .map_err(|e| Error::VideoReadError(format!("ffmpeg codec context error: {e}")))?;
        let decoder = context_decoder
            .decoder()
            .video()
            .map_err(|e| Error::VideoReadError(format!("ffmpeg decoder error: {e}")))?;

        let scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            // Flag has no effect here since we're not scaling resolution (only converting color format),
            // but is required by the API
            Flags::BILINEAR,
        )
        .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler error: {e}")))?;

        Ok(Self { context, video_stream_index, total_frames, fps, decoder, scaler })
    }
}

pub fn read_video_frames_ffmpeg(
    FFmpegVideoReader { mut context, video_stream_index, total_frames:_, fps: _, mut decoder, mut scaler}: FFmpegVideoReader,
) -> VideoResult<Vec<FFmpegFrame>> {
    let mut frames: Vec<FFmpegFrame> = Vec::new();

    for (stream, packet) in context.packets() {
        if stream.index() == video_stream_index {
            decoder
                .send_packet(&packet)
                .map_err(|e| Error::VideoReadError(format!("ffmpeg send packet error: {e}")))?;

            let mut decoded = ffmpeg_next::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = ffmpeg_next::util::frame::video::Video::empty();
                scaler
                    .run(&decoded, &mut rgb_frame)
                    .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler run error: {e}")))?;
                frames.push(FFmpegFrame::new(rgb_frame));
            }
        }
    }

    decoder.send_eof().map_err(|e| Error::VideoReadError(format!("ffmpeg send eof error: {e}")))?;
    let mut decoded = ffmpeg_next::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut rgb_frame = ffmpeg_next::util::frame::video::Video::empty();
        scaler
            .run(&decoded, &mut rgb_frame)
            .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler run error: {e}")))?;
        frames.push(FFmpegFrame::new(rgb_frame));
        decoded = ffmpeg_next::util::frame::video::Video::empty();
    }

    Ok(frames)
}

/// Converts a ffmpeg frame into ascii representation 2-d Vector
#[inline]
pub fn convert_ffmpeg_video_to_ascii(frame: &FFmpegFrame, config: &VideoConfig) -> Vec<Vec<&'static str>> {
    let scale_down = config.scale_down;
    let height_sample_scale = config.height_sample_scale;

    let width = frame.width;
    let height = frame.height;

    let scaled_width = (width as f32 / scale_down) as usize;
    let scaled_height = ((height as f32 / scale_down) / height_sample_scale) as usize;
    let mut res = vec![vec![" "; scaled_width]; scaled_height];

    let greyscale_ramp: &[&str] = if config.invert { &REVERSE_GREYSCALE_RAMP } else { &GREYSCALE_RAMP };

    res.par_iter_mut().enumerate().for_each(|(y, row)| {
        (0..scaled_width).for_each(|x| {
            let src_y = (y as f32 * scale_down * height_sample_scale) as u32;
            let src_x = (x as f32 * scale_down) as u32;

            let (r, g, b) = frame.get_pixel(src_x, src_y);
            let greyscale_value =
                RGB_TO_GREYSCALE.0 * r as f32 + RGB_TO_GREYSCALE.1 * g as f32 + RGB_TO_GREYSCALE.2 * b as f32;
            let index = (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
            row[x] = greyscale_ramp[index];
        })
    });

    res
}

pub struct FFmpegVideoWriter {
    pub context: ffmpeg_next::format::context::Output,
    pub stream_index: usize,
    pub stream_time_base: ffmpeg_next::Rational,
    pub encoder: ffmpeg_next::codec::encoder::video::Encoder,
    pub scaler: Context,
    pub width: u32,
    pub height: u32,
    pub frame_index: i64,
    closed: bool,
}

impl FFmpegVideoWriter {
    pub fn new(output_path: &str, width: u32, height: u32, fps: i32, bitrate: usize) -> VideoResult<Self> {
        ffmpeg_next::init().map_err(|e| Error::VideoWriteError(format!("ffmpeg init error: {e}")))?;

        let mut output =
            output(output_path).map_err(|e| Error::VideoWriteError(format!("ffmpeg output creation error: {e}")))?;

        let codec = ffmpeg_next::codec::encoder::find(Id::H264)
            .ok_or_else(|| Error::VideoWriteError("ffmpeg error: H264 codec not found".to_string()))?;

        let context_encoder = ffmpeg_next::codec::context::Context::new_with_codec(codec);
        let mut video_encoder = context_encoder
            .encoder()
            .video()
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg encoder error: {e}")))?;

        video_encoder.set_width(width);
        video_encoder.set_height(height);
        video_encoder.set_format(Pixel::YUV420P);
        video_encoder.set_frame_rate(Some(ffmpeg_next::Rational::new(fps, 1)));
        video_encoder.set_time_base(ffmpeg_next::Rational::new(1, fps));
        video_encoder.set_bit_rate(bitrate);
        video_encoder.set_gop(250);

        if output.format().flags().contains(ffmpeg_next::format::flag::Flags::GLOBAL_HEADER) {
            video_encoder.set_flags(ffmpeg_next::codec::flag::Flags::GLOBAL_HEADER);
        }

        let encoder =
            video_encoder.open().map_err(|e| Error::VideoWriteError(format!("ffmpeg encoder open error: {e}")))?;

        let mut stream =
            output.add_stream(codec).map_err(|e| Error::VideoWriteError(format!("ffmpeg add stream error: {e}")))?;
        stream.set_parameters(&encoder);
        stream.set_time_base(ffmpeg_next::Rational::new(1, fps));
        let stream_index = stream.index();
        let stream_time_base = stream.time_base();
        drop(stream);

        // Flag has no effect here since we're not scaling resolution (only converting color format),
        // but is required by the API
        let scaler = Context::get(Pixel::RGB24, width, height, Pixel::YUV420P, width, height, Flags::BILINEAR)
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg scaler error: {e}")))?;

        output.write_header().map_err(|e| Error::VideoWriteError(format!("ffmpeg write header error: {e}")))?;

        Ok(Self {
            context: output,
            stream_index,
            stream_time_base,
            encoder,
            scaler,
            width,
            height,
            frame_index: 0,
            closed: false,
        })
    }

    fn flush_packets(&mut self) -> VideoResult<()> {
        let mut packet = ffmpeg_next::codec::packet::Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.stream_index);
            packet.rescale_ts(self.encoder.time_base(), self.stream_time_base);
            packet
                .write_interleaved(&mut self.context)
                .map_err(|e| Error::VideoWriteError(format!("ffmpeg write interleaved error: {e}")))?;
            packet = ffmpeg_next::codec::packet::Packet::empty();
        }
        Ok(())
    }

    pub fn close(&mut self) -> VideoResult<()> {
        if self.closed {
            return Ok(());
        }

        self.encoder.send_eof().map_err(|e| Error::VideoWriteError(format!("ffmpeg send eof error: {e}")))?;

        self.flush_packets()?;

        self.context.write_trailer().map_err(|e| Error::VideoWriteError(format!("ffmpeg write trailer error: {e}")))?;
        self.closed = true;

        Ok(())
    }
}

impl Drop for FFmpegVideoWriter {
    fn drop(&mut self) {
        if let Err(e) = self.close() {
            eprintln!("FFmpegVideoWriter close error: {e}");
        }
    }
}

pub fn encode_ascii_frame_ffmpeg(
    config: &VideoConfig,
    ascii: &[Vec<&str>],
    width: u32,
    height: u32,
    writer: &mut FFmpegVideoWriter,
) -> VideoResult<FfmpegVideoFrame> {
    let frame = generate_ascii_image(ascii, width, height, config.invert, config.font_size);

    let mut rgb_frame = FfmpegVideoFrame::new(Pixel::RGB24, width, height);
    let stride = rgb_frame.stride(0);
    let data = rgb_frame.data_mut(0);

    for (y, row) in frame.enumerate_rows() {
        for (x, (_, _, pixel)) in row.enumerate() {
            let offset = y as usize * stride + x * 3;
            data[offset] = pixel[0];
            data[offset + 1] = pixel[1];
            data[offset + 2] = pixel[2];
        }
    }

    let mut yuv_frame = FfmpegVideoFrame::new(Pixel::YUV420P, width, height);
    yuv_frame.set_pts(Some(writer.frame_index));
    writer.frame_index += 1;

    writer
        .scaler
        .run(&rgb_frame, &mut yuv_frame)
        .map_err(|e| Error::VideoWriteError(format!("ffmpeg scaler run error: {e}")))?;

    Ok(yuv_frame)
}

#[inline]
pub fn write_to_ascii_video_ffmpeg(writer: &mut FFmpegVideoWriter, yuv_frame: &FfmpegVideoFrame) -> VideoResult<()> {
    writer
        .encoder
        .send_frame(yuv_frame)
        .map_err(|e| Error::VideoWriteError(format!("ffmpeg send frame error: {e}")))?;

    writer.flush_packets()?;

    Ok(())
}
