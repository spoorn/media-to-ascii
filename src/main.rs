use derive_builder::Builder;
use image::GenericImageView;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use opencv::prelude::*;
use opencv::core::{Mat, MatTraitConst};
use opencv::videoio;
use opencv::videoio::{CAP_ANY, VideoCaptureTrait};

static RGB_TO_GREYSCALE: (f32, f32, f32) = (0.299, 0.587, 0.114);

#[derive(Builder, Debug)]
#[builder(default)]
struct Config {
    image_path: String,
    scale_down: f32,
    height_sample_scale: f32,
    invert: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            image_path: "".to_string(),
            scale_down: 1.0,
            // 2.4 seems to be a good default
            height_sample_scale: 2.4,
            invert: false,
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
    max_fps: u64
}

impl Default for VideoConfig {
    fn default() -> Self {
        VideoConfig {
            video_path: "".to_string(),
            scale_down: 1.0,
            // 2.4 seems to be a good default
            height_sample_scale: 2.4,
            invert: false,
            max_fps: 10
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

fn write_to_file<S: AsRef<str>>(output_file: S, overwrite: bool, ascii: &Vec<Vec<&str>>) {
    let output_file = output_file.as_ref();
    if !overwrite && Path::new(output_file).exists() {
        panic!("File at {} already exists", output_file);
    }

    // TODO: change to create_new
    let file_option = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file);

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

fn process_image(config: Config) -> Vec<Vec<&'static str>> {
    let img_path = config.image_path.as_str();
    let scale_down = config.scale_down;
    let height_sample_scale = config.height_sample_scale;

    // From http://paulbourke.net/dataformats/asciiart/
    // let mut greyscale_ramp: Vec<&str> = vec![
    //     "$", "@", "B", "%", "8", "&", "W", "M", "#", "*", "o", "a", "h", "k", "b", "d", "p", "q",
    //     "w", "m", "Z", "O", "0", "Q", "L", "C", "J", "U", "Y", "X", "z", "c", "v", "u", "n", "x",
    //     "r", "j", "f", "t", "/", "\\", "|", "(", ")", "1", "{", "}", "[", "]", "?", "-", "_", "+",
    //     "~", "<", ">", "i", "!", "l", "I", ";", ":", ",", "\"", "^", "`", "'", ".", " ",
    // ];
    // Custom toned down greyscale ramp that seems to produce better images visually
    let mut greyscale_ramp: Vec<&str> = vec![
        "@", "B", "&", "#", "G", "P", "5", "J", "Y", "7", "?", "~", "!", ":", "^", ".", " ",
    ];

    // Invert greyscale, for dark backgrounds
    if config.invert {
        greyscale_ramp.reverse();
    }

    let img =
        image::open(img_path).expect(format!("Image at {img_path} could not be opened").as_str());
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
                let index =
                    (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
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

    let mut greyscale_ramp: Vec<&str> = vec![
        "@", "B", "&", "#", "G", "P", "5", "J", "Y", "7", "?", "~", "!", ":", "^", ".", " ",
    ];

    // Invert greyscale, for dark backgrounds
    if config.invert {
        greyscale_ramp.reverse();
    }

    for y in 0..res.len() {
        for x in 0..scaled_width {
            let pix: opencv::core::Vec3b = *frame.at_2d::<opencv::core::Vec3b>(
                (y as f32 * scale_down * height_sample_scale) as i32,
                (x as f32 * scale_down) as i32,
            ).unwrap();
            let greyscale_value = RGB_TO_GREYSCALE.0 * pix[0] as f32
                + RGB_TO_GREYSCALE.1 * pix[1] as f32
                + RGB_TO_GREYSCALE.2 * pix[2] as f32;
            let index =
                (greyscale_value * (greyscale_ramp.len() - 1) as f32 / 255.0).ceil() as usize;
            res[y][x] = greyscale_ramp[index];
        }
    }
    
    res
}

/// Processes video
/// 
/// References https://github.com/luketio/asciiframe/blob/7f23d8843278ad9cd4b53ff7110005aceeec1fcb/src/renderer.rs#L69.
fn process_video(config: VideoConfig) {
    let video_path = config.video_path.as_str();
    
    let mut capture = videoio::VideoCapture::from_file(video_path, CAP_ANY).expect(format!("Could not open video file at {video_path}").as_str());
    let num_frames = capture.get(videoio::CAP_PROP_FRAME_COUNT).unwrap() as u64;
    let orig_fps = capture.get(videoio::CAP_PROP_FPS).unwrap();
    let frame_time = 1.0 / orig_fps;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let clear_command = format!("{esc}c", esc = 27 as char);
    
    let frame_cut = orig_fps as u64 / config.max_fps;
    for i in 0..num_frames {
        let start = SystemTime::now();
        let mut frame = Mat::default();

        // CV_8UC3
        // TODO: error handling
        let read = capture.read(&mut frame).expect("Could not read frame of video");
        if !read {
            continue
        }

        if i % frame_cut == 0 {
            let ascii = convert_opencv_video(&frame, &config);

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

fn main() {
    // Note: Rust plugin can expand procedural macros using https://github.com/intellij-rust/intellij-rust/issues/6908
    // let config = ConfigBuilder::default()
    //     .image_path("/mnt/c/Users/Mikur/Downloads/giggles-closeup_orig.jpg".to_string())
    //     .scale_down(4.0)
    //     .invert(true)
    //     .build()
    //     .unwrap();
    // 
    // let ascii = process_image(config);
    // 
    // let output_file = Some("test.txt");
    // if let Some(file) = output_file {
    //     write_to_file(file, true, &ascii);
    // } else {
    //     print_ascii(&ascii);
    // }
    
    let video_config = VideoConfigBuilder::default()
        .video_path("/mnt/c/Users/Mikur/Desktop/download.mp4".to_string())
        .scale_down(4.0)
        .invert(true)
        .build()
        .unwrap();
    
    process_video(video_config);
}
