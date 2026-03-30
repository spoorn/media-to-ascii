use crate::image::generate_ascii_image;
use crate::util::UnsafeMat;
use crate::util::constants::{
    DARK_BGR_SCALAR, GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE, WHITE_BGR_SCALAR,
};
use crate::video::errors::Error;
use crate::video::{VideoConfig, VideoResult};
use opencv::core::{CV_8UC3, Mat, MatTraitConst, MatTraitManual, Vec3b};
use opencv::hub_prelude::{VideoCaptureTraitConst, VideoWriterTrait};
use opencv::videoio;
use opencv::videoio::VideoWriter;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub struct OpenCVVideoReader {
    pub capture: videoio::VideoCapture,
    pub total_frames: u64,
    pub fps: f64,
}
impl OpenCVVideoReader {
    pub fn new(video_path: &str) -> VideoResult<Self> {
        let capture = videoio::VideoCapture::from_file(video_path, videoio::CAP_ANY)
            .map_err(|e| Error::VideoReadError(format!("Could not open video file at {video_path}: {e}")))?;
        let total_frames = capture
            .get(videoio::CAP_PROP_FRAME_COUNT)
            .map_err(|e| Error::VideoReadError(format!("Could not get number of frames: {e}")))?
            as u64;
        let fps =
            capture.get(videoio::CAP_PROP_FPS).map_err(|e| Error::VideoReadError(format!("Could not get fps: {e}")))?;

        Ok(Self { capture, total_frames, fps })
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

pub fn encode_ascii_frame_opencv(config: &VideoConfig, ascii: &[Vec<&str>], width: u32, height: u32) -> Mat {
    let frame = generate_ascii_image(ascii, width, height, config.invert, config.font_size);
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

    opencv_frame
}

#[inline]
pub fn write_to_ascii_video_opencv(video_writer: &mut VideoWriter, opencv_frame: &Mat) {
    video_writer.write(opencv_frame).expect("Could not write frame to video");
}
