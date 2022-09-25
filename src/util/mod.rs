use opencv::core::{Size, Size_};

use crate::util::constants::{FONT_HEIGHT, MAGIC_HEIGHT_TO_WIDTH_RATIO};

pub mod constants;
pub mod file_util;

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

pub fn get_size_from_ascii(ascii: &[Vec<&str>]) -> Size_<i32> {
    Size::new(
        (ascii[0].len() as f32 * FONT_HEIGHT / MAGIC_HEIGHT_TO_WIDTH_RATIO) as i32,
        (ascii.len() as f32 * FONT_HEIGHT) as i32,
    )
}
