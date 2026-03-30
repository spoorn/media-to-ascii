use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use crate::util::ascii_to_str;
use crate::util::constants::MAGIC_HEIGHT_TO_WIDTH_RATIO;
use crate::util::file_util::{check_file_exists, check_valid_file};
use crate::video::encoder::Encoder;
use crate::video::errors::Error;
use crate::video::ffmpeg::FFmpegVideoReader;
use crate::video::opencv::{OpenCVVideoReader, OpenCVVideoWriter};
use crate::video::reader::Reader;
use crate::video::writer::Writer;
use crate::video::FFmpegVideoWriter;
use derive_builder::Builder;
use indicatif::ProgressBar;
use serde::Deserialize;

/// We track progress percentage in a global static mut as we only support 1 job at a time right now
pub static mut PROGRESS_PERCENTAGE: f32 = 0.0;

/// Current frame being processed during read step
pub static mut READ_CURRENT_FRAME: u64 = 0;
/// Current frame being processed during encode step
pub static mut ENCODE_CURRENT_FRAME: u64 = 0;
/// Current frame being processed during write step
pub static mut WRITE_CURRENT_FRAME: u64 = 0;

/// Total number of frames in the video
pub static mut TOTAL_FRAMES: u64 = 0;

/// Flag to signal cancellation of video processing
pub static mut CANCEL_REQUESTED: bool = false;

pub type VideoResult<T> = Result<T, crate::video::errors::Error>;

#[derive(Builder, Debug, Deserialize)]
#[builder(default)]
pub struct VideoConfig {
    /// Input Video file
    pub video_path: String,
    /// Multiplier to scale down input dimensions by when converting to ASCII.  For large frames,
    /// recommended to scale down more so output file size is more reasonable.  Affects output quality.
    /// Note: the output dimensions will also depend on the `font-size` setting.
    pub scale_down: f32,
    /// Font size of the ascii characters.  Defaults to 12.0.  Affects output quality.
    /// This directly affects the scaling of the output resolution as we "expand" each pixel to fit
    /// the Cascadia font to this size.  Note: this is not in "pixels" per-se, but will roughly scale
    /// the output to a multiple of this.
    pub font_size: f32,
    /// Rate at which we sample from the pixel rows of the frames.  This affects how stretched the
    /// output ascii is vertically due to discrepancies in the width-to-height ratio of the
    /// Cascadia font, and the input/output media dimensions.
    /// This essentially lets you shrink/squeeze the ascii text horizontally, without affecting
    /// output frame resolution.
    /// If you see text overflowing to the right of the output frame(s), or cut off short, you can
    /// try tuning this setting.  Larger values stretch the output. The default magic number is 2.046.
    /// See https://github.com/spoorn/media-to-ascii/issues/2 for in-depth details.
    pub height_sample_scale: f32,
    pub invert: bool,
    /// Max FPS for video outputs.  If outputting to video file, `use_max_fps_for_output_video`
    /// must be set to `true` to honor this setting.  Ascii videos in the terminal default to
    /// max_fps=10 for smoother visuals.
    pub max_fps: u64,
    /// Bitrate for video output, when using ffmpeg
    pub bitrate: Option<u64>,
    /// Output file path.  If omitted, output will be written to console.
    /// Supports most image formats, and .mp4 video outputs.
    /// Images will be resized to fit the ascii text.  Videos will honor the aspect ratio of the
    /// input, but resolution will be scaled differently approximately to `(height|width) / scale_down * font_size`.
    pub output_video_path: Option<String>,
    /// Overwrite any output file if it already exists
    pub overwrite: bool,
    /// Use the max_fps setting for video file outputs.
    pub use_max_fps_for_output_video: bool,
    /// Rotate the input (0 = 90 CLOCKWISE, 1 = 180, 2 = 90 COUNTER-CLOCKWISE)
    pub rotate: i32,
    pub should_rotate: bool,
    pub use_opencv: bool,
    // /// Number of threads for parallel processing during encode step. [default: number of logical CPU cores]
    // pub num_threads: u8,
}

impl Default for VideoConfig {
    fn default() -> Self {
        VideoConfig {
            video_path: "".to_string(),
            scale_down: 1.0,
            font_size: 12.0,
            height_sample_scale: MAGIC_HEIGHT_TO_WIDTH_RATIO,
            invert: false,
            max_fps: 10,
            bitrate: None,
            output_video_path: None,
            overwrite: false,
            use_max_fps_for_output_video: false,
            rotate: -1,
            should_rotate: false,
            use_opencv: false,
            // num_threads: available_parallelism().unwrap().get() as u8,
        }
    }
}

pub enum VideoReader {
    OpenCV(OpenCVVideoReader),
    FFmpeg(FFmpegVideoReader),
}
impl Reader for VideoReader {
    fn total_frames(&self) -> u64 {
        match self {
            VideoReader::OpenCV(e) => e.total_frames(),
            VideoReader::FFmpeg(e) => e.total_frames(),
        }
    }

    fn fps(&self) -> f64 {
        match self {
            VideoReader::OpenCV(e) => e.fps(),
            VideoReader::FFmpeg(e) => e.fps(),
        }
    }

    fn read_frame(&mut self, config: &VideoConfig) -> VideoResult<()> {
        match self {
            VideoReader::OpenCV(e) => e.read_frame(config),
            VideoReader::FFmpeg(e) => e.read_frame(config),
        }
    }

    fn read_frame_as_ascii(&mut self, config: &VideoConfig) -> VideoResult<Vec<Vec<&str>>> {
        match self {
            VideoReader::OpenCV(e) => e.read_frame_as_ascii(config),
            VideoReader::FFmpeg(e) => e.read_frame_as_ascii(config),
        }
    }

    fn finish(&mut self) -> VideoResult<()> {
        match self {
            VideoReader::OpenCV(e) => e.finish(),
            VideoReader::FFmpeg(e) => e.finish(),
        }
    }
}

pub enum VideoWriter {
    OpenCV(OpenCVVideoWriter),
    FFmpeg(FFmpegVideoWriter),
}
impl TryFrom<(&VideoConfig, VideoReader)> for VideoWriter {
    type Error = Error;

    fn try_from((config, reader): (&VideoConfig, VideoReader)) -> Result<Self, Self::Error> {
        match reader {
            VideoReader::OpenCV(e) => Ok(VideoWriter::OpenCV(OpenCVVideoWriter::new(&config, e)?)),
            VideoReader::FFmpeg(e) => Ok(VideoWriter::FFmpeg(FFmpegVideoWriter::new(&config, e)?)),
        }
    }
}
impl Encoder for VideoWriter {
    fn encode_frame(&mut self, config: &VideoConfig, frame_index: usize) -> VideoResult<()> {
        match self {
            VideoWriter::OpenCV(e) => e.encode_frame(config, frame_index),
            VideoWriter::FFmpeg(e) => e.encode_frame(config, frame_index),
        }
    }
}
impl Writer for VideoWriter {
    fn write_frame(&mut self, frame_index: usize) -> VideoResult<()> {
        match self {
            VideoWriter::OpenCV(e) => e.write_frame(frame_index),
            VideoWriter::FFmpeg(e) => e.write_frame(frame_index),
        }
    }

    fn close(&mut self) -> VideoResult<()> {
        match self {
            VideoWriter::OpenCV(e) => e.close(),
            VideoWriter::FFmpeg(e) => e.close(),
        }
    }
}

/// Processes video
///
/// References https://github.com/luketio/asciiframe/blob/7f23d8843278ad9cd4b53ff7110005aceeec1fcb/src/renderer.rs#L69.
pub fn process_video(mut config: VideoConfig) -> VideoResult<()> {
    // Reset cancellation flag and progress tracking
    unsafe {
        CANCEL_REQUESTED = false;
        READ_CURRENT_FRAME = 0;
        ENCODE_CURRENT_FRAME = 0;
        WRITE_CURRENT_FRAME = 0;
        TOTAL_FRAMES = 0;
        PROGRESS_PERCENTAGE = 0.0;
    }

    eprintln!("Processing video with config: {config:#?}");

    let video_path = config.video_path.as_str();
    check_valid_file(video_path);

    let output_video_path = config.output_video_path.as_ref();
    let output_video_file: bool = output_video_path.is_some();

    if output_video_file {
        check_file_exists(output_video_path.unwrap(), config.overwrite);
    }

    let mut reader = if config.use_opencv {
        VideoReader::OpenCV(OpenCVVideoReader::new(video_path)?)
    } else {
        VideoReader::FFmpeg(FFmpegVideoReader::new(video_path)?)
    };

    let num_frames = reader.total_frames();
    unsafe {
        TOTAL_FRAMES = num_frames;
    }

    if num_frames == 0 {
        return Ok(());
    }

    let orig_fps = reader.fps();
    let frame_time = 1.0 / orig_fps;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let clear_command = format!("{esc}c", esc = 27 as char);

    let frame_cut = orig_fps as u64 / config.max_fps;
    config.should_rotate = config.rotate > -1 && config.rotate < 3;

    if output_video_file {
        eprintln!("Encoding video from {} to ascii video at {}", video_path, output_video_path.unwrap());

        // let pool = ThreadPoolBuilder::new()
        //     .num_threads(config.num_threads as usize) // tune this based on available RAM
        //     .build()
        //     .unwrap();

        // Triple progress bar for reading, encoding, then writing frames
        let progressbar = ProgressBar::new(num_frames * 3);

        (0..num_frames)
            .into_iter()
            .map(|i| {
                // Process first frame to get output dimensions and initialize video writer
                eprintln!("Reading frame {} of {num_frames}", i + 1);

                unsafe {
                    if CANCEL_REQUESTED {
                        CANCEL_REQUESTED = false;
                        return Err(Error::Cancelled);
                    }
                    READ_CURRENT_FRAME += 1;
                }

                reader.read_frame(&config).inspect_err(|_e| {
                    eprintln!("Error reading frame {i} from input video",);
                })?;

                progressbar.inc(1);
                unsafe {
                    PROGRESS_PERCENTAGE = progressbar.position() as f32 / progressbar.length().unwrap() as f32;
                }

                Ok(())
            })
            .collect::<VideoResult<()>>()?;

        reader.finish()?;

        let mut writer = VideoWriter::try_from((&config, reader))?;

        unsafe {
            ENCODE_CURRENT_FRAME = 1;
        }

        progressbar.inc(1);
        unsafe {
            PROGRESS_PERCENTAGE = progressbar.position() as f32 / progressbar.length().unwrap() as f32;
        }

        //pool.install(|| {
        (1..num_frames)
            .into_iter()
            .filter_map(|i| {
                // eprintln to prevent buffering issues with rayon
                eprintln!("Encoding frame {} of {num_frames}", i + 1);
                //std::io::stdout().flush().unwrap();

                if config.use_max_fps_for_output_video && i % frame_cut == 0 {
                    return None;
                }

                // Check for cancellation
                unsafe {
                    if CANCEL_REQUESTED {
                        return Some(Err(Error::Cancelled));
                    }
                    ENCODE_CURRENT_FRAME += 1;
                }

                let res = writer.encode_frame(&config, i as usize);
                progressbar.inc(1);
                unsafe {
                    PROGRESS_PERCENTAGE = progressbar.position() as f32 / progressbar.length().unwrap() as f32;
                }
                Some(res)
            })
            .collect::<VideoResult<()>>()?;
        //})?;

        unsafe {
            if CANCEL_REQUESTED {
                return Err(Error::Cancelled);
            }
        }

        for i in 0..num_frames {
            eprintln!("Writing frame {} of {num_frames}", i + 1);

            if config.use_max_fps_for_output_video && i as u64 % frame_cut == 0 {
                continue;
            }

            unsafe {
                if CANCEL_REQUESTED {
                    return Err(Error::Cancelled);
                }
                WRITE_CURRENT_FRAME += 1;
            }

            writer.write_frame(i as usize)?;

            progressbar.inc(1);
            unsafe {
                PROGRESS_PERCENTAGE = progressbar.position() as f32 / progressbar.length().unwrap() as f32;
            }
        }

        // Writes the video explicitly just for clarity
        writer.close()?;
        progressbar.finish();
        unsafe {
            PROGRESS_PERCENTAGE = 1.0;
        }

        eprintln!("Finished writing output video file to {}", output_video_path.unwrap());
    } else {
        for i in 0..num_frames {
            let start = SystemTime::now();

            let ascii = reader.read_frame_as_ascii(&config)?;

            // Write to terminal

            if i % frame_cut == 0 {
                let ascii_str = ascii_to_str(&ascii);
                write!(handle, "{}", clear_command).unwrap();
                write!(handle, "{}", ascii_str).unwrap();
            }

            let elapsed = start.elapsed().unwrap().as_secs_f64();
            if elapsed < frame_time {
                sleep(Duration::from_millis(((frame_time - elapsed) * 1000.0) as u64));
            }
        }
    }

    Ok(())
}
