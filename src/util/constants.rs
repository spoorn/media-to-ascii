use once_cell::sync::Lazy;
use rusttype::{Font, Scale};

pub static RGB_TO_GREYSCALE: (f32, f32, f32) = (0.299, 0.587, 0.114);
// Font height of ascii when producing videos, approximately the number of pixels
pub static FONT_HEIGHT: f32 = 12.0;
pub static FONT_SCALE: Scale = Scale {
    x: FONT_HEIGHT,
    y: FONT_HEIGHT,
};
// When creating the output ascii video, for Cascadia font, this is a magic height to width ratio
// for the video dimensions so the text fits to the frames' ends
pub static MAGIC_HEIGHT_TO_WIDTH_RATIO: f32 = 2.048;
pub static CASCADIA_FONT: Lazy<Font<'static>> = Lazy::new(|| {
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
pub static GREYSCALE_RAMP: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        " ", ".", "^", ":", "~", "?", "7", "Y", "J", "5", "P", "G", "#", "&", "B", "@",
    ]
});

pub static REVERSE_GREYSCALE_RAMP: Lazy<Vec<&str>> = Lazy::new(|| {
    let mut clone = GREYSCALE_RAMP.clone();
    clone.reverse();
    clone
});
