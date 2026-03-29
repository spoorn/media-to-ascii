use opencv::core::{MatTraitConst, Vec3b};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use crate::util::constants::{GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE};
use crate::util::UnsafeMat;
use crate::video::VideoConfig;

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