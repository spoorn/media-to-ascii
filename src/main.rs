use clap::{ArgGroup, Parser};

use crate::image::{ImageConfigBuilder, process_image};
use crate::video::{process_video, VideoConfigBuilder};

mod util;
mod image;
mod video;

/// Converts media (images and videos) to ascii, and displays output either as an output media file
/// or in the terminal.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("input_path")
        .required(true)
        .multiple(false)
        .args(&["image_path", "video_path"]),
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

fn main() {
    // if let Ok(mut file) = File::open(Path::new("output.mp4")) {
    //     let mut buffer = [0u8; 1024];
    //     let mut output = File::create("outputtest.mp4").unwrap();
    //
    //     loop {
    //         match file.read(&mut buffer) {
    //             Ok(size) => {
    //                 if size == 0 {
    //                     println!("Finished");
    //                     break;
    //                 }
    //                 println!("read {size} bytes");
    //                 if let Err(e) = output.write_all(&buffer[0..size]) {
    //                     eprintln!("Error: {e}");
    //                 }
    //             }
    //             Err(e) => {
    //                 eprintln!("Error: {e}");
    //             }
    //         }
    //     }
    // }

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
