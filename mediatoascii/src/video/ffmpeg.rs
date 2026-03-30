use ffmpeg_next::codec::encoder::Video as VideoEncoder;
use ffmpeg_next::codec::Id;
use ffmpeg_next::format::{input, output, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video as FfmpegVideoFrame;
use image::Rgb;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::image::generate_ascii_image;
use crate::util::constants::{GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE};
use crate::util::FFmpegFrame;
use crate::video::errors::Error;
use crate::video::VideoConfig;
use crate::video::VideoResult;

pub fn read_video_frames_ffmpeg(path: &str) -> VideoResult<Vec<FFmpegFrame>> {
    ffmpeg_next::init().map_err(|e| Error::VideoReadError(format!("ffmpeg init error: {e}")))?;

    let mut ictx = input(path).map_err(|e| Error::VideoReadError(format!("ffmpeg input error: {e}")))?;

    let video_stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or_else(|| Error::VideoReadError("ffmpeg error: no video stream found".to_string()))?;
    let video_stream_index = video_stream.index();

    let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
        .map_err(|e| Error::VideoReadError(format!("ffmpeg codec context error: {e}")))?;
    let mut decoder =
        context_decoder.decoder().video().map_err(|e| Error::VideoReadError(format!("ffmpeg decoder error: {e}")))?;

    let mut scaler = Context::get(
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

    let mut frames: Vec<FFmpegFrame> = Vec::new();

    for (stream, packet) in ictx.packets() {
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
    pub stream: ffmpeg_next::Stream,
    pub encoder: VideoEncoder,
    pub scaler: Context,
    pub width: u32,
    pub height: u32,
}

impl FFmpegVideoWriter {
    pub fn new(output_path: &str, width: u32, height: u32, fps: f32, bitrate: usize) -> VideoResult<Self> {
        ffmpeg_next::init().map_err(|e| Error::VideoWriteError(format!("ffmpeg init error: {e}")))?;

        let mut output = output(output_path, "mp4")
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg output creation error: {e}")))?;

        let codec = ffmpeg_next::codec::encoder::find(Id::H264)
            .ok_or_else(|| Error::VideoWriteError("ffmpeg error: H264 codec not found".to_string()))?;

        let mut encoder = VideoEncoder::new(codec);
        encoder.set_width(width);
        encoder.set_height(height);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_frame_rate(ffmpeg_next::Rational::new(fps as i32, 1));
        encoder.set_bit_rate(bitrate);
        encoder.set_gop(250);

        let encoder = encoder.open().map_err(|e| Error::VideoWriteError(format!("ffmpeg encoder open error: {e}")))?;

        let mut stream = output.add_stream(codec);
        stream.set_parameters(encoder.as_ref());
        stream.set_time_base(ffmpeg_next::Rational::new(1, fps as i32));

        // Flag has no effect here since we're not scaling resolution (only converting color format),
        // but is required by the API
        let scaler = Context::get(Pixel::RGB24, width, height, Pixel::YUV420P, width, height, Flags::BILINEAR)
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg scaler error: {e}")))?;

        Ok(Self { context: output, stream, encoder, scaler, width, height })
    }

    fn flush_packets(&mut self) -> VideoResult<()> {
        let mut packet = ffmpeg_next::codec::packet::Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream_index(self.stream.index());
            packet.rescale_ts(ffmpeg_next::Rational::new(1, 1), self.stream.time_base());
            self.context
                .write(&packet, self.stream.index())
                .map_err(|e| Error::VideoWriteError(format!("ffmpeg write error: {e}")))?;
            packet = ffmpeg_next::codec::packet::Packet::empty();
        }
        Ok(())
    }

    pub fn close(&mut self) -> VideoResult<()> {
        self.encoder.send_frame(None).map_err(|e| Error::VideoWriteError(format!("ffmpeg send eof error: {e}")))?;

        self.flush_packets()?;

        self.context.write_trailer().map_err(|e| Error::VideoWriteError(format!("ffmpeg write trailer error: {e}")))?;

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
) -> VideoResult<()> {
    let frame = generate_ascii_image(ascii, width, height, config.invert, config.font_size);

    let mut rgb_frame = FfmpegVideoFrame::new(ffmpeg_next::util::format::Pixel::RGB24, width, height);

    for (y, row) in frame.enumerate_rows() {
        let row_pixels: Vec<Rgb<u8>> = row.map(|(_, _, pix)| Rgb([pix[0], pix[1], pix[2]])).collect();
        for (x, pixel) in row_pixels.iter().enumerate() {
            let offset = ((y as u32 * width + x as u32) * 3) as usize;
            rgb_frame.data_mut(0)[offset] = pixel[0];
            rgb_frame.data_mut(0)[offset + 1] = pixel[1];
            rgb_frame.data_mut(0)[offset + 2] = pixel[2];
        }
    }

    let mut yuv_frame = FfmpegVideoFrame::new(ffmpeg_next::util::format::Pixel::YUV420P, width, height);

    writer
        .scaler
        .run(&rgb_frame, &mut yuv_frame)
        .map_err(|e| Error::VideoWriteError(format!("ffmpeg scaler run error: {e}")))?;

    writer
        .encoder
        .send_frame(Some(&yuv_frame))
        .map_err(|e| Error::VideoWriteError(format!("ffmpeg send frame error: {e}")))?;

    writer.flush_packets()?;

    Ok(())
}
