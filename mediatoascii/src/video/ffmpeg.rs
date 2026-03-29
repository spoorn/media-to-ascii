use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::media::Type;
use ffmpeg_next::software::scaling::{context::Context, flag::Flags};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

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
        // Placeholder flag for when we want to scale resolution in the future
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
