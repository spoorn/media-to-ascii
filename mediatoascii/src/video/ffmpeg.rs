use ffmpeg_next::codec::Id;
use ffmpeg_next::format::{input, output, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use ffmpeg_next::util::frame::video::Video as FfmpegVideoFrame;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::image::generate_ascii_image;
use crate::util::constants::{GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE};
use crate::util::{get_size_from_ascii, FFmpegFrame};
use crate::video::encoder::Encoder;
use crate::video::errors::Error;
use crate::video::reader::Reader;
use crate::video::writer::Writer;
use crate::video::VideoConfig;
use crate::video::VideoResult;

/// We scale the time base and frame index as low values seem to skew ffmpeg's internal timestamp
/// calculations and cause weird things like make a 2 second 30fps video output 15360 FPS for 4ms
const TIME_BASE_SCALE: i32 = 1000;

pub struct FFmpegVideoReader {
    pub context: ffmpeg_next::format::context::Input,
    pub video_stream_index: usize,
    total_frames: u64,
    fps: f64,
    frames: Vec<FFmpegFrame>,
    decoder: ffmpeg_next::codec::decoder::video::Video,
    scaler: Context,
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

        Ok(Self {
            context,
            video_stream_index,
            total_frames,
            fps,
            frames: Vec::with_capacity(total_frames as usize),
            decoder,
            scaler,
        })
    }

    fn read_single_frame(&mut self) -> VideoResult<FFmpegFrame> {
        let mut decoded = ffmpeg_next::util::frame::video::Video::empty();

        loop {
            // Try to receive a frame first
            if self.decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = ffmpeg_next::util::frame::video::Video::empty();
                self.scaler
                    .run(&decoded, &mut rgb_frame)
                    .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler run error: {e}")))?;
                return Ok(FFmpegFrame::new(rgb_frame));
            }

            // Otherwise, feed more packets
            match self.context.packets().next() {
                Some((stream, packet)) => {
                    if stream.index() == self.video_stream_index {
                        self.decoder
                            .send_packet(&packet)
                            .map_err(|e| Error::VideoReadError(format!("send packet error: {e}")))?;
                    }
                }
                None => {
                    // EOF, just continue until caller calls finish()
                    return Ok(FFmpegFrame::default());
                }
            }
        }
    }
}

impl Reader for FFmpegVideoReader {
    fn total_frames(&self) -> u64 {
        self.total_frames
    }

    fn fps(&self) -> f64 {
        self.fps
    }

    fn read_frame(&mut self, _config: &VideoConfig) -> VideoResult<()> {
        let frame = self.read_single_frame()?;
        if !frame.is_empty() {
            self.frames.push(frame);
        }
        Ok(())
    }

    fn read_frame_as_ascii(&mut self, config: &VideoConfig) -> VideoResult<Vec<Vec<&str>>> {
        let frame = self.read_single_frame()?;
        Ok(convert_ffmpeg_video_to_ascii(&frame, &config))
    }

    fn finish(&mut self) -> VideoResult<()> {
        self.decoder.send_eof().map_err(|e| Error::VideoReadError(format!("ffmpeg send eof error: {e}")))?;
        let mut decoded = ffmpeg_next::util::frame::video::Video::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = ffmpeg_next::util::frame::video::Video::empty();
            self.scaler
                .run(&decoded, &mut rgb_frame)
                .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler run error: {e}")))?;
            self.frames.push(FFmpegFrame::new(rgb_frame));
            decoded = ffmpeg_next::util::frame::video::Video::empty();
        }

        Ok(())
    }
}

pub fn read_video_frames_ffmpeg(
    FFmpegVideoReader { mut context, video_stream_index, total_frames:_, fps: _, mut frames, mut decoder, mut scaler}: FFmpegVideoReader,
) -> VideoResult<()> {
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

    Ok(())
}

pub struct FFmpegVideoWriter {
    pub context: ffmpeg_next::format::context::Output,
    pub stream_index: usize,
    pub stream_time_base: ffmpeg_next::Rational,
    pub encoder: ffmpeg_next::codec::encoder::video::Encoder,
    pub scaler: Context,
    input_frames: Vec<FFmpegFrame>,
    width: u32,
    height: u32,
    frames: Vec<FfmpegVideoFrame>,
    pub frame_index: i64,
    closed: bool,
}

impl FFmpegVideoWriter {
    pub fn new(config: &VideoConfig, reader: FFmpegVideoReader) -> VideoResult<Self> {
        ffmpeg_next::init().map_err(|e| Error::VideoWriteError(format!("ffmpeg init error: {e}")))?;

        let mut output = output(config.output_video_path.as_ref().unwrap())
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg output creation error: {e}")))?;

        let codec = ffmpeg_next::codec::encoder::find(Id::H264)
            .ok_or_else(|| Error::VideoWriteError("ffmpeg error: H264 codec not found".to_string()))?;

        let context_encoder = ffmpeg_next::codec::context::Context::new_with_codec(codec);

        let mut video_encoder = context_encoder
            .encoder()
            .video()
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg encoder error: {e}")))?;

        let mut frames = Vec::with_capacity(reader.total_frames as usize);
        let input_frames = reader.frames;
        let ascii = convert_ffmpeg_video_to_ascii(&input_frames[0], &config);
        let (width, height) = get_size_from_ascii(&ascii, config.height_sample_scale, config.font_size);
        // ffmpeg for h264 requires width/height to be divisible by 2
        let width = if width % 2 == 0 { width } else { width + 1 };
        let height = if height % 2 == 0 { height } else { height + 1 };
        // if width * height > 9437184 {
        //     // a / b = width / height
        //     // a * b <= 9437184
        //     return Err(Error::ResolutionTooLarge);
        // }

        let time_base = ffmpeg_next::Rational::new(1, reader.fps as i32 * TIME_BASE_SCALE);
        println!("fps: {}, time_base: {}/{}", reader.fps, time_base.numerator(), time_base.denominator());

        video_encoder.set_width(width);
        video_encoder.set_height(height);
        video_encoder.set_format(Pixel::YUV420P);
        video_encoder.set_frame_rate(Some(ffmpeg_next::Rational::new(reader.fps as i32, 1)));
        video_encoder.set_time_base(time_base);
        if let Some(bitrate) = config.bitrate {
            video_encoder.set_bit_rate(bitrate as usize);
        }
        video_encoder.set_gop(250);

        if output.format().flags().contains(ffmpeg_next::format::flag::Flags::GLOBAL_HEADER) {
            video_encoder.set_flags(ffmpeg_next::codec::flag::Flags::GLOBAL_HEADER);
        }

        let encoder =
            video_encoder.open().map_err(|e| Error::VideoWriteError(format!("ffmpeg encoder open error: {e:?}")))?;

        let mut stream =
            output.add_stream(codec).map_err(|e| Error::VideoWriteError(format!("ffmpeg add stream error: {e}")))?;
        stream.set_parameters(&encoder);
        stream.set_time_base(time_base);
        let stream_index = stream.index();
        let stream_time_base = stream.time_base();
        drop(stream);

        // Flag has no effect here since we're not scaling resolution (only converting color format),
        // but is required by the API
        let mut scaler = Context::get(Pixel::RGB24, width, height, Pixel::YUV420P, width, height, Flags::BILINEAR)
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg scaler error: {e}")))?;

        output.write_header().map_err(|e| Error::VideoWriteError(format!("ffmpeg write header error: {e}")))?;

        frames.push(encode_ascii_frame_ffmpeg(&config, &ascii, width, height, 0, &mut scaler)?);

        Ok(Self {
            context: output,
            stream_index,
            stream_time_base,
            encoder,
            scaler,
            input_frames,
            width,
            height,
            frames,
            frame_index: 1,
            closed: false,
        })
    }

    fn flush_packets(&mut self) -> VideoResult<()> {
        let mut packet = ffmpeg_next::codec::packet::Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.stream_index);
            packet.rescale_ts(self.encoder.time_base(), self.stream_time_base);
            packet
                .write(&mut self.context)
                .map_err(|e| Error::VideoWriteError(format!("ffmpeg write interleaved error: {e}")))?;
            packet = ffmpeg_next::codec::packet::Packet::empty();
        }
        Ok(())
    }
}

impl Encoder for FFmpegVideoWriter {
    fn encode_frame(&mut self, config: &VideoConfig, frame_index: usize) -> VideoResult<()> {
        let ascii = convert_ffmpeg_video_to_ascii(&self.input_frames[frame_index], &config);
        let frame =
            encode_ascii_frame_ffmpeg(&config, &ascii, self.width, self.height, self.frame_index, &mut self.scaler)?;
        self.frame_index += 1;
        self.frames.push(frame);
        Ok(())
    }
}

impl Writer for FFmpegVideoWriter {
    fn write_frame(&mut self, frame_index: usize) -> VideoResult<()> {
        self.encoder
            .send_frame(&self.frames[frame_index])
            .map_err(|e| Error::VideoWriteError(format!("ffmpeg send frame error: {e}")))?;

        self.flush_packets()?;

        Ok(())
    }

    fn close(&mut self) -> VideoResult<()> {
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

pub fn encode_ascii_frame_ffmpeg(
    config: &VideoConfig,
    ascii: &[Vec<&str>],
    width: u32,
    height: u32,
    frame_index: i64,
    scaler: &mut Context,
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
    yuv_frame.set_pts(Some(frame_index * TIME_BASE_SCALE as i64));

    scaler
        .run(&rgb_frame, &mut yuv_frame)
        .map_err(|e| Error::VideoWriteError(format!("ffmpeg scaler run error: {e}")))?;

    Ok(yuv_frame)
}
