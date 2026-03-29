use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};

use crate::util::FFmpegFrame;
use crate::video::errors::Error;
use crate::video::VideoResult;

pub fn read_video_frames_ffmpeg(path: &str) -> VideoResult<Vec<FFmpegFrame>> {
    ffmpeg::init().map_err(|e| Error::VideoReadError(format!("ffmpeg init error: {e}")))?;

    let mut ictx = input(path).map_err(|e| Error::VideoReadError(format!("ffmpeg input error: {e}")))?;

    let video_stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or_else(|| Error::VideoReadError("ffmpeg error: no video stream found".to_string()))?;
    let video_stream_index = video_stream.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
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

            let mut decoded = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut rgb_frame = ffmpeg::util::frame::video::Video::empty();
                scaler
                    .run(&decoded, &mut rgb_frame)
                    .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler run error: {e}")))?;
                frames.push(FFmpegFrame::new(rgb_frame));
            }
        }
    }

    decoder.send_eof().map_err(|e| Error::VideoReadError(format!("ffmpeg send eof error: {e}")))?;
    let mut decoded = ffmpeg::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut rgb_frame = ffmpeg::util::frame::video::Video::empty();
        scaler
            .run(&decoded, &mut rgb_frame)
            .map_err(|e| Error::VideoReadError(format!("ffmpeg scaler run error: {e}")))?;
        frames.push(FFmpegFrame::new(rgb_frame));
    }

    Ok(frames)
}
