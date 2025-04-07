use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use crate::image::generate_ascii_image;
use crate::util::constants::{
    DARK_BGR_SCALAR, GREYSCALE_RAMP, MAGIC_HEIGHT_TO_WIDTH_RATIO, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE,
    WHITE_BGR_SCALAR,
};
use crate::util::file_util::{check_file_exists, check_valid_file};
use crate::util::{UnsafeMat, ascii_to_str, get_size_from_ascii};
use crate::video::errors::Error;
use derive_builder::Builder;
use indicatif::ProgressBar;
use opencv::core::{CV_8UC3, Mat, MatTraitConst, MatTraitManual, Size, Vec3b};
use opencv::videoio;
use opencv::videoio::{CAP_ANY, VideoCaptureTrait, VideoCaptureTraitConst, VideoWriter, VideoWriterTrait};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::Deserialize;

/// We track progress percentage in a global static mut as we only support 1 job at a time right now
pub static mut PROGRESS_PERCENTAGE: f32 = 0.0;

pub type VideoResult<T> = Result<T, crate::video::errors::Error>;

#[derive(Builder, Debug, Deserialize)]
#[builder(default)]
pub struct VideoConfig {
    /// Input Video file
    video_path: String,
    /// Multiplier to scale down input dimensions by when converting to ASCII.  For large frames,
    /// recommended to scale down more so output file size is more reasonable.  Affects output quality.
    /// Note: the output dimensions will also depend on the `font-size` setting.
    scale_down: f32,
    /// Font size of the ascii characters.  Defaults to 12.0.  Affects output quality.
    /// This directly affects the scaling of the output resolution as we "expand" each pixel to fit
    /// the Cascadia font to this size.  Note: this is not in "pixels" per-se, but will roughly scale
    /// the output to a multiple of this.
    font_size: f32,
    /// Rate at which we sample from the pixel rows of the frames.  This affects how stretched the
    /// output ascii is vertically due to discrepancies in the width-to-height ratio of the
    /// Cascadia font, and the input/output media dimensions.
    /// This essentially lets you shrink/squeeze the ascii text horizontally, without affecting
    /// output frame resolution.
    /// If you see text overflowing to the right of the output frame(s), or cut off short, you can
    /// try tuning this setting.  Larger values stretch the output. The default magic number is 2.046.
    /// See https://github.com/spoorn/media-to-ascii/issues/2 for in-depth details.
    height_sample_scale: f32,
    invert: bool,
    /// Max FPS for video outputs.  If outputting to video file, `use_max_fps_for_output_video`
    /// must be set to `true` to honor this setting.  Ascii videos in the terminal default to
    /// max_fps=10 for smoother visuals.
    max_fps: u64,
    /// Output file path.  If omitted, output will be written to console.
    /// Supports most image formats, and .mp4 video outputs.
    /// Images will be resized to fit the ascii text.  Videos will honor the aspect ratio of the
    /// input, but resolution will be scaled differently approximately to `(height|width) / scale_down * font_size`.
    output_video_path: Option<String>,
    /// Overwrite any output file if it already exists
    overwrite: bool,
    /// Use the max_fps setting for video file outputs.
    use_max_fps_for_output_video: bool,
    /// Rotate the input (0 = 90 CLOCKWISE, 1 = 180, 2 = 90 COUNTER-CLOCKWISE)
    rotate: i32,
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
            output_video_path: None,
            overwrite: false,
            use_max_fps_for_output_video: false,
            rotate: -1,
        }
    }
}

/// Converts an opencv frame Matrix into ascii representation 2-d Vector
///
/// References https://github.com/luketio/asciiframe/blob/main/src/converter.rs#L15.
#[inline]
pub fn convert_opencv_video_to_ascii(frame: &UnsafeMat, config: &VideoConfig) -> Vec<Vec<&'static str>> {
    let scale_down = config.scale_down;
    let height_sample_scale = config.height_sample_scale;

    let width = frame.cols();
    let height = frame.rows();
    //println!("width: {}, height: {}", width, height);
    // TODO: scaled dims
    let scaled_width = (width as f32 / scale_down) as usize;
    let scaled_height = ((height as f32 / scale_down) / height_sample_scale) as usize;
    //println!("scaled scaled_width: {}, scaled_height: {}", scaled_width, scaled_height);
    let mut res = vec![vec![" "; scaled_width]; scaled_height];

    // Invert greyscale, for dark backgrounds
    let greyscale_ramp: &[&str] = if config.invert { &REVERSE_GREYSCALE_RAMP } else { &GREYSCALE_RAMP };

    // SAFETY: operates pixels independently
    res.par_iter_mut().enumerate().for_each(|(y, row)| {
        // TODO: is this parallelizable?
        // TODO: This is a bad sampling method when scaling down
        (0..scaled_width).for_each(|x| {
            let pix: &Vec3b = frame
                .at_2d::<Vec3b>((y as f32 * scale_down * height_sample_scale) as i32, (x as f32 * scale_down) as i32)
                .unwrap();
            let greyscale_value = RGB_TO_GREYSCALE.0 * pix[0] as f32
                + RGB_TO_GREYSCALE.1 * pix[1] as f32
                + RGB_TO_GREYSCALE.2 * pix[2] as f32;
            let index = (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
            row[x] = greyscale_ramp[index];
        })
    });

    res
}

#[inline]
pub fn write_to_ascii_video(config: &VideoConfig, ascii: &[Vec<&str>], video_writer: &mut VideoWriter, size: &Size) {
    let frame = generate_ascii_image(ascii, size, config.invert, config.font_size);
    //println!("image frame width: {}, height: {}", frame.width(), frame.height());

    // Create opencv CV_8UC3 frame
    // opencv uses BGR format
    let bgr_background_color = if config.invert { WHITE_BGR_SCALAR } else { DARK_BGR_SCALAR };

    let mut opencv_frame =
        Mat::new_rows_cols_with_default(frame.height() as i32, frame.width() as i32, CV_8UC3, bgr_background_color)
            .unwrap();

    // Writing per row is much faster than reading and writing each pixel
    frame.enumerate_rows().for_each(|(row, x)| {
        let row_pixels: Vec<Vec3b> = x.map(|(_, _, pix)| Vec3b::from([pix[2], pix[1], pix[0]])).collect();

        opencv_frame.at_row_mut::<Vec3b>(row as i32).unwrap().iter_mut().enumerate().for_each(|(i, pix)| {
            *pix = row_pixels[i];
        })
    });

    video_writer.write(&opencv_frame).expect("Could not write frame to video");
}

/// Processes video
///
/// References https://github.com/luketio/asciiframe/blob/7f23d8843278ad9cd4b53ff7110005aceeec1fcb/src/renderer.rs#L69.
pub fn process_video(config: VideoConfig) -> VideoResult<()> {
    let video_path = config.video_path.as_str();
    check_valid_file(video_path);

    let output_video_path = config.output_video_path.as_ref();
    let output_video_file: bool = output_video_path.is_some();

    if output_video_file {
        check_file_exists(output_video_path.unwrap(), config.overwrite);
    }

    let mut capture = videoio::VideoCapture::from_file(video_path, CAP_ANY)
        .unwrap_or_else(|_| panic!("Could not open video file at {video_path}"));
    let num_frames = capture.get(videoio::CAP_PROP_FRAME_COUNT).unwrap() as u64;
    let orig_fps = capture.get(videoio::CAP_PROP_FPS).unwrap();
    let frame_time = 1.0 / orig_fps;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let clear_command = format!("{esc}c", esc = 27 as char);

    let frame_cut = orig_fps as u64 / config.max_fps;

    // Video output
    let mut video_writer: VideoWriter = VideoWriter::default().unwrap();
    let mut output_frame_size: Size = Size::default();
    let should_rotate = config.rotate > -1 && config.rotate < 3;

    if output_video_file {
        println!("Encoding video from {} to ascii video at {}", video_path, output_video_path.unwrap());
    }

    let progressbar = ProgressBar::new(num_frames);

    for i in 0..num_frames {
        println!("Processing frame {i} of {num_frames}");
        let start = SystemTime::now();
        let mut frame = UnsafeMat(Mat::default());

        // CV_8UC3
        // TODO: error handling
        let read = capture.read(&mut frame.0).expect("Could not read frame of video");
        if !read {
            eprintln!("Error reading frame {} from input video.  Skipping frame and continuing...", i);
            continue;
        }

        // Rotate
        if should_rotate {
            opencv::core::rotate(&frame.clone(), &mut frame.0, config.rotate).unwrap();
        }

        let ascii = convert_opencv_video_to_ascii(&frame, &config);

        if output_video_file {
            // Write to video file

            if i == 0 {
                // Initialize VideoWriter for real
                output_frame_size = get_size_from_ascii(&ascii, config.height_sample_scale, config.font_size);
                // Openh264 codec seems to have this dimension limitation so we cap it
                if output_frame_size.width * output_frame_size.height > 9437184 {
                    // a / b = width / height
                    // a * b <= 9437184
                    return Err(Error::ResolutionTooLarge);
                }
                //println!("frame size: {:?}", output_frame_size);
                let video_fps = if config.use_max_fps_for_output_video { config.max_fps as f64 } else { orig_fps };
                // TODO: allow any video output
                video_writer = VideoWriter::new(
                    output_video_path.unwrap().as_str(),
                    VideoWriter::fourcc('a', 'v', 'c', '1').unwrap(),
                    video_fps,
                    output_frame_size,
                    true,
                )
                .unwrap();
            }

            progressbar.inc(1);
            unsafe {
                PROGRESS_PERCENTAGE = progressbar.position() as f32 / progressbar.length().unwrap() as f32;
            }
            if config.use_max_fps_for_output_video && i % frame_cut == 0 {
                continue;
            }

            write_to_ascii_video(&config, &ascii, &mut video_writer, &output_frame_size);
        } else {
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

    // Writes the video explicitly just for clarity
    video_writer.release().unwrap();
    progressbar.finish();
    unsafe {
        PROGRESS_PERCENTAGE = 1.0;
    }

    if output_video_file {
        println!("Finished writing output video file to {}", output_video_path.unwrap());
    }

    Ok(())
}
