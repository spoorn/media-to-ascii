use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};

use image::{ImageBuffer, Rgb};
use opencv::core::{Mat, Size, Size_};

use crate::util::constants::FONT_HEIGHT;

pub mod constants;
pub mod file_util;

/// Wrapper around Mat that let's us bypass non-Sync since Mat uses *mut c_void ptr.  Tricks
/// compiler into letting us use this across threads even though it's unsafe.  Allows for
/// parallelization of some operations at very high performance.
pub struct UnsafeMat(pub Mat);
unsafe impl Sync for UnsafeMat {}
impl Deref for UnsafeMat {
    type Target = Mat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnsafeMat {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Wrapper around ImageBuffer that bypasses non-Sync.  Tricks compiler into letting us use this
/// across threads even though it's unsafe.  Allows for parallelization of some operations at very
/// high performance.
pub struct UnsafeImageBuffer(pub UnsafeCell<Option<ImageBuffer<Rgb<u8>, Vec<u8>>>>);
unsafe impl Sync for UnsafeImageBuffer {}
impl Deref for UnsafeImageBuffer {
    type Target = UnsafeCell<Option<ImageBuffer<Rgb<u8>, Vec<u8>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnsafeImageBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[inline]
pub fn ascii_to_str(ascii: &[Vec<&str>]) -> String {
    let mut buffer = String::default();
    for row in ascii {
        for s in row {
            buffer.push_str(s);
        }
        buffer.push('\n');
    }
    buffer
}

pub fn print_ascii(ascii: &[Vec<&str>]) {
    print!("{}", ascii_to_str(ascii));
}

pub fn get_size_from_ascii(ascii: &[Vec<&str>], height_sample_scale: f32) -> Size_<i32> {
    Size::new(
        (ascii[0].len() as f32 * FONT_HEIGHT / height_sample_scale) as i32,
        (ascii.len() as f32 * FONT_HEIGHT) as i32,
    )
}
