use derive_builder::Builder;
use image::GenericImageView;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

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
            height_sample_scale: 2.4,
            invert: false,
        }
    }
}

fn print_ascii(ascii: &Vec<Vec<&str>>) {
    for y in 0..ascii.len() {
        let row = &ascii[y];
        for x in 0..row.len() {
            print!("{}", row[x]);
        }
        println!();
    }
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

fn main() {
    // Note: Rust plugin can expand procedural macros using https://github.com/intellij-rust/intellij-rust/issues/6908
    let config = ConfigBuilder::default()
        .image_path("".to_string())
        .scale_down(1.0)
        .invert(true)
        .build()
        .unwrap();

    let ascii = process_image(config);

    let output_file = Some("test.txt");
    if let Some(file) = output_file {
        write_to_file(file, true, &ascii);
    } else {
        print_ascii(&ascii);
    }
}
