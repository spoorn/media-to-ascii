use derive_builder::Builder;
use image::{GenericImageView, ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use once_cell::sync::Lazy;
use opencv::core::{Mat, MatTraitConst, Scalar, Size, Size_, CV_8UC3};
use opencv::prelude::*;
use opencv::videoio;
use opencv::videoio::{VideoCaptureTrait, VideoWriter, CAP_ANY};
use rusttype::{Font, Scale};
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

static RGB_TO_GREYSCALE: (f32, f32, f32) = (0.299, 0.587, 0.114);
// Font height of ascii when producing videos, approximately the number of pixels
static FONT_HEIGHT: f32 = 12.0;
static FONT_SCALE: Scale = Scale {
    x: FONT_HEIGHT,
    y: FONT_HEIGHT,
};
// When creating the output ascii video, for Cascadia font, this is a magic height to width ratio
// for the video dimensions so the text fits to the frames' ends
static MAGIC_HEIGHT_TO_WIDTH_RATIO: f32 = 2.048;
static CASCADIA_FONT: Lazy<Font<'static>> = Lazy::new(|| {
    let font_data = include_bytes!("fonts/Cascadia.ttf");
    Font::try_from_bytes(font_data).unwrap()
});

// From http://paulbourke.net/dataformats/asciiart/
// let mut greyscale_ramp: Vec<&str> = vec![
//     "$", "@", "B", "%", "8", "&", "W", "M", "#", "*", "o", "a", "h", "k", "b", "d", "p", "q",
//     "w", "m", "Z", "O", "0", "Q", "L", "C", "J", "U", "Y", "X", "z", "c", "v", "u", "n", "x",
//     "r", "j", "f", "t", "/", "\\", "|", "(", ")", "1", "{", "}", "[", "]", "?", "-", "_", "+",
//     "~", "<", ">", "i", "!", "l", "I", ";", ":", ",", "\"", "^", "`", "'", ".", " ",
// ];
// Custom toned down greyscale ramp that seems to produce better images visually
static GREYSCALE_RAMP: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "@", "B", "&", "#", "G", "P", "5", "J", "Y", "7", "?", "~", "!", ":", "^", ".", " ",
    ]
});

static REVERSE_GREYSCALE_RAMP: Lazy<Vec<&str>> = Lazy::new(|| {
    let mut clone = GREYSCALE_RAMP.clone();
    clone.reverse();
    clone
});

#[derive(Builder, Debug)]
#[builder(default)]
struct ImageConfig {
    image_path: String,
    scale_down: f32,
    height_sample_scale: f32,
    invert: bool,
    output_file_path: Option<String>,
    output_image_path: Option<String>,
    overwrite: bool,
}

impl Default for ImageConfig {
    fn default() -> Self {
        ImageConfig {
            image_path: "".to_string(),
            scale_down: 1.0,
            // 2.4 seems to be a good default
            height_sample_scale: 2.4,
            invert: false,
            output_file_path: None,
            output_image_path: None,
            overwrite: false,
        }
    }
}

#[derive(Builder, Debug)]
#[builder(default)]
struct VideoConfig {
    video_path: String,
    scale_down: f32,
    height_sample_scale: f32,
    invert: bool,
    max_fps: u64,
    output_video_path: Option<String>,
    overwrite: bool,
    use_max_fps_for_output_video: bool,
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
            output_video_path: Some("output.mp4".to_string()),
            overwrite: false,
            use_max_fps_for_output_video: false,
        }
    }
}

fn ascii_to_str(ascii: &Vec<Vec<&str>>) -> String {
    let mut buffer = String::default();
    for y in 0..ascii.len() {
        let row = &ascii[y];
        for x in 0..row.len() {
            buffer.push_str(row[x]);
        }
        buffer.push('\n');
    }
    buffer
}

fn print_ascii(ascii: &Vec<Vec<&str>>) {
    print!("{}", ascii_to_str(ascii));
}

fn check_file_exists<S: AsRef<str>>(file: S, overwrite: bool) {
    let file = file.as_ref();
    if !overwrite && Path::new(file).exists() {
        panic!("File at {} already exists", file);
    }
}

fn write_to_file<S: AsRef<str>>(output_file: S, overwrite: bool, ascii: &Vec<Vec<&str>>) {
    let output_file = output_file.as_ref();
    check_file_exists(output_file, overwrite);

    // TODO: change to create_new
    let file_option = OpenOptions::new().write(true).create(true).truncate(true).open(output_file);

    match file_option {
        Ok(mut file) => {
            for y in 0..ascii.len() {
                let row = &ascii[y];
                file.write(row.join("").as_bytes()).unwrap();
                file.write("\r\n".as_bytes()).unwrap();
            }
        }
        Err(_) => {
            panic!("Could not write output to file {}", output_file);
        }
    }
}

#[inline(always)]
fn generate_ascii_image(ascii: &Vec<Vec<&str>>, size: &Size, invert: bool) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let background_color = if invert { Rgb([40u8, 42u8, 54u8]) } else { Rgb([255u8, 255u8, 255u8]) };
    let text_color = if invert { Rgb([255u8, 255u8, 255u8]) } else { Rgb([0u8, 0u8, 0u8]) };
    let mut frame = RgbImage::from_pixel(size.width as u32, size.height as u32, background_color);

    for row in 0..ascii.len() {
        let text_row = ascii[row].join("");
        draw_text_mut(
            &mut frame,
            text_color,
            0,
            (row as f32 * FONT_HEIGHT) as i32,
            FONT_SCALE,
            &CASCADIA_FONT,
            text_row.as_str(),
        );
    }

    frame
}

fn get_size_from_ascii(ascii: &Vec<Vec<&str>>) -> Size_<i32> {
    Size::new(
        (ascii[0].len() as f32 * FONT_HEIGHT / MAGIC_HEIGHT_TO_WIDTH_RATIO) as i32,
        (ascii.len() as f32 * FONT_HEIGHT) as i32,
    )
}

fn write_to_image<S: AsRef<str>>(output_file: S, overwrite: bool, ascii: &Vec<Vec<&str>>, size: &Size, invert: bool) {
    let output_file = output_file.as_ref();
    check_file_exists(output_file, overwrite);
    generate_ascii_image(ascii, size, invert).save(output_file).unwrap();
}

fn process_image(config: &ImageConfig) -> Vec<Vec<&'static str>> {
    let img_path = config.image_path.as_str();
    let scale_down = config.scale_down;
    let height_sample_scale = config.height_sample_scale;

    // Invert greyscale, for dark backgrounds
    let greyscale_ramp: &Vec<&str> = if config.invert { &REVERSE_GREYSCALE_RAMP } else { &GREYSCALE_RAMP };

    let img = image::open(img_path).expect(format!("Image at {img_path} could not be opened").as_str());
    let (width, height) = img.dimensions();
    let scaled_width = (width as f32 / scale_down) as usize;
    let scaled_height = ((height as f32 / scale_down) / height_sample_scale) as usize;

    let mut res = vec![vec![" "; scaled_width]; scaled_height];

    for y in 0..res.len() {
        for x in 0..scaled_width {
            let pix = img.get_pixel(
                (x as f32 * scale_down) as u32,
                (y as f32 * scale_down * height_sample_scale) as u32,
            );
            if pix[3] != 0 {
                let greyscale_value = RGB_TO_GREYSCALE.0 * pix[0] as f32
                    + RGB_TO_GREYSCALE.1 * pix[1] as f32
                    + RGB_TO_GREYSCALE.2 * pix[2] as f32;
                let index = (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
                res[y][x] = greyscale_ramp[index];
            }
        }
    }

    res
}

/// Converts an opencv frame Matrix into ascii representation 2-d Vector
///
/// References https://github.com/luketio/asciiframe/blob/main/src/converter.rs#L15.
fn convert_opencv_video(frame: &Mat, config: &VideoConfig) -> Vec<Vec<&'static str>> {
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

    for y in 0..res.len() {
        for x in 0..scaled_width {
            let pix: opencv::core::Vec3b = *frame
                .at_2d::<opencv::core::Vec3b>(
                    (y as f32 * scale_down * height_sample_scale) as i32,
                    (x as f32 * scale_down) as i32,
                )
                .unwrap();
            let greyscale_value = RGB_TO_GREYSCALE.0 * pix[0] as f32
                + RGB_TO_GREYSCALE.1 * pix[1] as f32
                + RGB_TO_GREYSCALE.2 * pix[2] as f32;
            let index = (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
            res[y][x] = greyscale_ramp[index];
        }
    }

    res
}

fn write_to_ascii_video(config: &VideoConfig, ascii: &Vec<Vec<&str>>, video_writer: &mut VideoWriter, size: &Size) {
    let frame = generate_ascii_image(ascii, size, config.invert);

    // Create opencv CV_8UC3 frame
    // opencv uses BGR format
    let bgr_background_color =
        if config.invert { Scalar::from((54.0, 42.0, 40.0)) } else { Scalar::from((255.0, 255.0, 255.0)) };
    let mut opencv_frame = Mat::new_rows_cols_with_default(
        frame.height() as i32,
        frame.width() as i32,
        CV_8UC3,
        bgr_background_color,
    )
    .unwrap();

    let (width, height) = frame.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pix = frame.get_pixel(x, y);
            // RGB to BGR
            *opencv_frame.at_2d_mut::<opencv::core::Vec3b>(y as i32, x as i32).unwrap() =
                opencv::core::Vec3b::from([pix[2], pix[1], pix[0]]);
        }
    }

    video_writer.write(&opencv_frame).expect("Could not write frame to video");
}

/// Processes video
///
/// References https://github.com/luketio/asciiframe/blob/7f23d8843278ad9cd4b53ff7110005aceeec1fcb/src/renderer.rs#L69.
fn process_video(config: VideoConfig) {
    let video_path = config.video_path.as_str();
    let output_video_path = config.output_video_path.as_ref();
    check_file_exists(output_video_path.unwrap(), config.overwrite);

    let mut capture = videoio::VideoCapture::from_file(video_path, CAP_ANY)
        .expect(format!("Could not open video file at {video_path}").as_str());
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
    println!("Num Frames: {}", num_frames);

    for i in 0..num_frames {
        let start = SystemTime::now();
        let mut frame = Mat::default();

        // CV_8UC3
        // TODO: error handling
        let read = capture.read(&mut frame).expect("Could not read frame of video");
        if !read {
            continue;
        }

        let ascii = convert_opencv_video(&frame, &config);

        if output_video_path.is_some() {
            // Write to video file

            if i == 0 {
                // Initialize VideoWriter for real
                output_frame_size = get_size_from_ascii(&ascii);
                video_writer = VideoWriter::new(
                    output_video_path.unwrap().as_str(),
                    VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
                    orig_fps,
                    output_frame_size,
                    true,
                )
                .unwrap();
            }

            if config.use_max_fps_for_output_video && i % frame_cut == 0 {
                continue;
            }

            println!("Writing frame {}", i);
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
}

fn main() {
    // Note: Rust plugin can expand procedural macros using https://github.com/intellij-rust/intellij-rust/issues/6908
    // let config = ImageConfigBuilder::default()
    //     .image_path("".to_string())
    //     .scale_down(4.0)
    //     .invert(true)
    //     .build()
    //     .unwrap();
    //
    // let ascii = process_image(&config);
    //
    // if let Some(file) = config.output_file_path.as_ref() {
    //     write_to_file(file, config.overwrite.clone(), &ascii);
    // }
    //
    // if let Some(file) = config.output_image_path.as_ref() {
    //     write_to_image(file, config.overwrite.clone(), &ascii, &get_size_from_ascii(&ascii), config.invert.clone());
    // }
    //
    // if config.output_file_path.is_none() && config.output_image_path.is_none() {
    //     print_ascii(&ascii);
    // }

    let video_config = VideoConfigBuilder::default()
        .video_path("".to_string())
        .scale_down(2.0)
        .invert(true)
        .build()
        .unwrap();

    process_video(video_config);
}
