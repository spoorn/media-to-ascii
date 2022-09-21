use clap::{AppSettings, ArgGroup, Parser};
use derive_builder::Builder;
use image::{GenericImageView, ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use indicatif::ProgressBar;
use once_cell::sync::Lazy;
use opencv::core::{Mat, MatTraitConst, Scalar, Size, Size_, Vec3b, CV_8UC3};
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
        " ", ".", "^", ":", "~", "?", "7", "Y", "J", "5", "P", "G", "#", "&", "B", "@",
    ]
});

static REVERSE_GREYSCALE_RAMP: Lazy<Vec<&str>> = Lazy::new(|| {
    let mut clone = GREYSCALE_RAMP.clone();
    clone.reverse();
    clone
});

/// Converts media (images and videos) to ascii, and displays output either as an output media file
/// or in the terminal.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(group(
    ArgGroup::new("input_path")
        .required(true)
        .multiple(false)
        .args(&["image-path", "video-path"]),
))]
struct Cli {
    /// Input Image file.  One of image_path, or video_path must be populated.
    #[clap(long, value_parser)]
    image_path: Option<String>,
    /// Input Video file.  One of image_path, or video_path must be populated.
    #[clap(long, value_parser)]
    video_path: Option<String>,
    /// Multiplier to scale down input dimensions by when converting to ASCII.  For large frames,
    /// recommended to scale down more so output file size is more reasonable.
    #[clap(long, default_value_t = 1.0, value_parser)]
    scale_down: f32,
    /// Rate at which we sample from the pixel rows of the frames.  This affects how stretched the
    /// output ascii is in the vertical or y-axis.
    #[clap(long, default_value_t = 2.4, value_parser)]
    height_sample_scale: f32,
    /// Invert ascii greyscale ramp (For light backgrounds.  Default OFF is for dark backgrounds.)
    #[clap(short, long, action)]
    invert: bool,
    /// Overwrite any output file if it already exists
    #[clap(long, action)]
    overwrite: bool,
    /// Max FPS for video outputs.  If outputting to video file, `use_max_fps_for_output_video`
    /// must be set to `true` to honor this setting.  Ascii videos in the terminal default to
    /// max_fps=10 for smoother visuals.
    #[clap(long, value_parser)]
    max_fps: Option<u64>,
    /// For images, if output_file_path is specified, will save the ascii text as-is to the output
    /// rather than an image file.
    #[clap(long, action)]
    as_text: bool,
    /// Output file path.  If omitted, output will be written to console.
    /// Supports most image formats, and .mp4 video outputs.
    #[clap(short, long, value_parser)]
    output_file_path: Option<String>,
    /// Use the max_fps setting for video file outputs.
    #[clap(long, action)]
    use_max_fps_for_output_video: bool,
    /// Rotate the input (0 = 90 CLOCKWISE, 1 = 180, 2 = 90 COUNTER-CLOCKWISE)
    #[clap(short, long, value_parser = clap::value_parser!(i32).range(0..3))]
    rotate: Option<i32>,
}

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

#[inline]
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
        panic!("File at {} already exists, and overwrite is set to false", file);
    }
}

fn check_valid_file<S: AsRef<str>>(path: S) {
    let path = path.as_ref();
    if !Path::new(path).is_file() {
        panic!("Path at {} is not a valid file!", path)
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

#[inline]
fn generate_ascii_image(ascii: &Vec<Vec<&str>>, size: &Size, invert: bool) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let background_color = if invert { Rgb([255u8, 255u8, 255u8]) } else { Rgb([40u8, 42u8, 54u8]) };
    let text_color = if invert { Rgb([0u8, 0u8, 0u8]) } else { Rgb([255u8, 255u8, 255u8]) };
    let mut frame = RgbImage::from_pixel(size.width as u32, size.height as u32, background_color);

    // parallel implementation
    // let mut ascii_strings: Vec<String> = vec![];
    //
    // ascii.par_iter().map(|row| {
    //     row.join("")
    // }).collect_into_vec(&mut ascii_strings);
    //
    // ascii_strings.iter().enumerate().for_each(|(row, text_row)| {
    //     draw_text_mut(
    //         &mut frame,
    //         text_color,
    //         0,
    //         (row as f32 * FONT_HEIGHT) as i32,
    //         FONT_SCALE,
    //         &CASCADIA_FONT,
    //         text_row.as_str(),
    //     );
    // });

    // let mut flat_ascii = vec![];
    // ascii.iter().for_each(|row| {
    //     flat_ascii.extend(row);
    // });
    ascii.iter().enumerate().for_each(|(row, row_data)| {
        let text_row = row_data.join("");
        draw_text_mut(
            &mut frame,
            text_color,
            0,
            (row as f32 * FONT_HEIGHT) as i32,
            FONT_SCALE,
            &CASCADIA_FONT,
            text_row.as_str(),
        );
    });

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
    match generate_ascii_image(ascii, size, invert).save(output_file) {
        Ok(_) => {
            println!("Successfully saved ascii image to {}", output_file);
        }
        Err(e) => {
            eprintln!("Failed to save ascii image to {}: {}", output_file, e);
        }
    }
}

#[inline]
fn convert_image_to_ascii(config: &ImageConfig) -> Vec<Vec<&'static str>> {
    let img_path = config.image_path.as_str();
    check_valid_file(img_path);
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

fn process_image(config: ImageConfig) {
    let ascii = convert_image_to_ascii(&config);

    if let Some(file) = config.output_file_path.as_ref() {
        write_to_file(file, config.overwrite.clone(), &ascii);
    }

    if let Some(file) = config.output_image_path.as_ref() {
        write_to_image(
            file,
            config.overwrite.clone(),
            &ascii,
            &get_size_from_ascii(&ascii),
            config.invert.clone(),
        );
    }

    if config.output_file_path.is_none() && config.output_image_path.is_none() {
        print_ascii(&ascii);
    }
}

/// Converts an opencv frame Matrix into ascii representation 2-d Vector
///
/// References https://github.com/luketio/asciiframe/blob/main/src/converter.rs#L15.
#[inline]
fn convert_opencv_video_to_ascii(frame: &Mat, config: &VideoConfig) -> Vec<Vec<&'static str>> {
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
            res[y][x] = greyscale_ramp[index];
        }
    }

    res
}

#[inline]
fn write_to_ascii_video(config: &VideoConfig, ascii: &Vec<Vec<&str>>, video_writer: &mut VideoWriter, size: &Size) {
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
fn process_video(config: VideoConfig) {
    let video_path = config.video_path.as_str();
    check_valid_file(video_path);

    let output_video_path = config.output_video_path.as_ref();
    let output_video_file: bool = output_video_path.is_some();

    if output_video_file {
        check_file_exists(output_video_path.unwrap(), config.overwrite);
    }

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

fn main() {
    let cli = Cli::parse();
    // Note: Rust plugin can expand procedural macros using https://github.com/intellij-rust/intellij-rust/issues/6908

    if let Some(image_path) = cli.image_path {
        let mut config_builder = ImageConfigBuilder::default();
        config_builder
            .image_path(image_path)
            .scale_down(cli.scale_down)
            .height_sample_scale(cli.height_sample_scale)
            .invert(cli.invert)
            .overwrite(cli.overwrite);

        if let Some(output_path) = cli.output_file_path {
            if cli.as_text {
                config_builder.output_file_path(Some(output_path));
            } else {
                config_builder.output_image_path(Some(output_path));
            }
        }

        let config = config_builder.build().unwrap();
        process_image(config);
    } else if let Some(video_path) = cli.video_path {
        let mut config_builder = VideoConfigBuilder::default();
        config_builder
            .video_path(video_path)
            .scale_down(cli.scale_down)
            .invert(cli.invert)
            .overwrite(cli.overwrite)
            .use_max_fps_for_output_video(cli.use_max_fps_for_output_video);

        if let Some(max_fps) = cli.max_fps {
            config_builder.max_fps(max_fps);
        }

        if let Some(output_path) = cli.output_file_path {
            config_builder.output_video_path(Some(output_path));
        }

        if let Some(rotate) = cli.rotate {
            config_builder.rotate(rotate);
        }

        let config = config_builder.build().unwrap();
        process_video(config);
    } else {
        panic!("Either image-path or video-path must be provided!");
    }
}
