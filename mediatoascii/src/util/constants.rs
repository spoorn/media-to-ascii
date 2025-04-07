use ab_glyph::FontRef;
use image::Rgb;
use opencv::core::Scalar;
use std::sync::LazyLock;

/// NTSC formula: https://en.wikipedia.org/wiki/Grayscale
pub const RGB_TO_GREYSCALE: (f32, f32, f32) = (0.299, 0.587, 0.114);
/// White RGB
pub static WHITE_RGB: Rgb<u8> = Rgb([255u8, 255u8, 255u8]);
/// Faded black RGB
pub static DARK_RGB: Rgb<u8> = Rgb([40u8, 42u8, 54u8]);
/// Black RGB
pub static BLACK_RGB: Rgb<u8> = Rgb([0u8, 0u8, 0u8]);
/// White BGR scalar (opencv uses BGR)
pub static WHITE_BGR_SCALAR: Scalar = Scalar::new(255.0, 255.0, 255.0, 0.0);
/// Faded black BGR scalar (opencv uses BGR)
pub static DARK_BGR_SCALAR: Scalar = Scalar::new(54.0, 42.0, 40.0, 0.0);

/// When creating the output ascii video, for Cascadia font, this is a magic height to width ratio
/// for the video dimensions so the text fits to the frames' ends
/// See https://github.com/spoorn/media-to-ascii/issues/2
pub const MAGIC_HEIGHT_TO_WIDTH_RATIO: f32 = 2.046;
pub static CASCADIA_FONT: LazyLock<FontRef> =
    LazyLock::new(|| FontRef::try_from_slice(include_bytes!("fonts/Cascadia.ttf")).unwrap());

// From http://paulbourke.net/dataformats/asciiart/
// let mut greyscale_ramp: Vec<&str> = vec![
//     "$", "@", "B", "%", "8", "&", "W", "M", "#", "*", "o", "a", "h", "k", "b", "d", "p", "q",
//     "w", "m", "Z", "O", "0", "Q", "L", "C", "J", "U", "Y", "X", "z", "c", "v", "u", "n", "x",
//     "r", "j", "f", "t", "/", "\\", "|", "(", ")", "1", "{", "}", "[", "]", "?", "-", "_", "+",
//     "~", "<", ">", "i", "!", "l", "I", ";", ":", ",", "\"", "^", "`", "'", ".", " ",
// ];
// Custom toned down greyscale ramp that seems to produce better images visually
pub const GREYSCALE_RAMP: &[&str] = &[" ", ".", "^", ":", "~", "?", "7", "Y", "J", "5", "P", "G", "#", "&", "B", "@"];

// No const reverse: https://github.com/rust-lang/rust/issues/100784
pub const REVERSE_GREYSCALE_RAMP: &[&str] =
    &["@", "B", "&", "#", "G", "P", "5", "J", "Y", "7", "?", "~", ":", "^", ".", " "];
