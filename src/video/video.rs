use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use derive_builder::Builder;
use indicatif::ProgressBar;
use opencv::core::{Mat, MatTraitConst, MatTraitManual, Scalar, Size, Vec3b, CV_8UC3};
use opencv::videoio;
use opencv::videoio::{VideoCaptureTrait, VideoCaptureTraitConst, VideoWriter, VideoWriterTrait, CAP_ANY};

use crate::image_to_ascii::generate_ascii_image;
use crate::util::constants::{GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE};
use crate::util::file_util::{check_file_exists, check_valid_file};
use crate::util::{ascii_to_str, get_size_from_ascii};

#[derive(Builder, Debug)]
#[builder(default)]
pub struct VideoConfig {
    video_path: String,
    scale_down: f32,
    height_sample_scale: f32,
    invert: bool,
    max_fps: u64,
    output_video_path: Option<String>,
    overwrite: bool,
    use_max_fps_for_output_video: bool,
    rotate: i32,
}

impl Default for VideoConfig {
    fn default() -> Self {
        VideoConfig {
            video_path: "".to_string(),
            scale_down: 1.0,
            // 2.4 seems to be a good default
            height_sample_scale: 2.4,
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
pub fn convert_opencv_video_to_ascii(frame: &Mat, config: &VideoConfig) -> Vec<Vec<&'static str>> {
    let scale_down = config.scale_down;
    let height_sample_scale = config.height_sample_scale;

    let width = frame.cols();
    let height = frame.rows();
    // TODO: scaled dims
    let scaled_width = (width as f32 / scale_down) as usize;
    let scaled_height = ((height as f32 / scale_down) / height_sample_scale) as usize;
    let mut res = vec![vec![" "; scaled_width]; scaled_height];

    // Invert greyscale, for dark backgrounds
    let greyscale_ramp: &Vec<&str> = if config.invert { &REVERSE_GREYSCALE_RAMP } else { &GREYSCALE_RAMP };

    for (y, row) in res.iter_mut().enumerate() {
        for x in 0..scaled_width {
            let pix: Vec3b = *frame
                .at_2d::<Vec3b>(
                    (y as f32 * scale_down * height_sample_scale) as i32,
                    (x as f32 * scale_down) as i32,
                )
                .unwrap();
            let greyscale_value = RGB_TO_GREYSCALE.0 * pix[0] as f32
                + RGB_TO_GREYSCALE.1 * pix[1] as f32
                + RGB_TO_GREYSCALE.2 * pix[2] as f32;
            let index = (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
            row[x] = greyscale_ramp[index];
        }
    }

    res
}

#[inline]
pub fn write_to_ascii_video(config: &VideoConfig, ascii: &[Vec<&str>], video_writer: &mut VideoWriter, size: &Size) {
    let frame = generate_ascii_image(ascii, size, config.invert);

    // Create opencv CV_8UC3 frame
    // opencv uses BGR format
    let bgr_background_color =
        if config.invert { Scalar::from((255.0, 255.0, 255.0)) } else { Scalar::from((54.0, 42.0, 40.0)) };

    let mut opencv_frame = Mat::new_rows_cols_with_default(
        frame.height() as i32,
        frame.width() as i32,
        CV_8UC3,
        bgr_background_color,
    )
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
pub fn process_video(config: VideoConfig) {
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
        println!(
            "Encoding video from {} to ascii video at {}",
            video_path,
            output_video_path.unwrap()
        );
    }

    let progressbar = ProgressBar::new(num_frames);

    for i in 0..num_frames {
        let start = SystemTime::now();
        let mut frame = Mat::default();

        // CV_8UC3
        // TODO: error handling
        let read = capture.read(&mut frame).expect("Could not read frame of video");
        if !read {
            continue;
        }

        // Rotate
        if should_rotate {
            opencv::core::rotate(&frame.clone(), &mut frame, config.rotate).unwrap();
        }

        let ascii = convert_opencv_video_to_ascii(&frame, &config);

        if output_video_file {
            // Write to video file

            if i == 0 {
                // Initialize VideoWriter for real
                output_frame_size = get_size_from_ascii(&ascii);
                let video_fps = if config.use_max_fps_for_output_video { config.max_fps as f64 } else { orig_fps };
                video_writer = VideoWriter::new(
                    output_video_path.unwrap().as_str(),
                    VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
                    video_fps,
                    output_frame_size,
                    true,
                )
                .unwrap();
            }

            progressbar.inc(1);
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

    if output_video_file {
        println!("Finished writing output video file to {}", output_video_path.unwrap());
    }
}
