use crate::image::generate_ascii_image;
use crate::util::constants::{
    DARK_BGR_SCALAR, GREYSCALE_RAMP, REVERSE_GREYSCALE_RAMP, RGB_TO_GREYSCALE, WHITE_BGR_SCALAR,
};
use crate::util::{UnsafeMat, get_size_from_ascii};
use crate::video::encoder::Encoder;
use crate::video::errors::Error;
use crate::video::reader::Reader;
use crate::video::writer::Writer;
use crate::video::{VideoConfig, VideoResult};
use opencv::core::{CV_8UC3, Mat, MatTraitConst, MatTraitManual, Size, Vec3b};
use opencv::hub_prelude::{VideoCaptureTraitConst, VideoWriterTrait};
use opencv::videoio;
use opencv::videoio::{VideoCaptureTrait, VideoWriter};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

pub struct OpenCVVideoReader {
    pub capture: videoio::VideoCapture,
    total_frames: u64,
    fps: f64,
    frames: Vec<UnsafeMat>,
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

        Ok(Self { capture, total_frames, fps, frames: Vec::with_capacity(total_frames as usize) })
    }

    fn read_single_frame(&mut self, config: &VideoConfig) -> VideoResult<UnsafeMat> {
        let mut frame = UnsafeMat(Mat::default());

        // CV_8UC3
        if !self.capture.read(&mut frame.0).expect("Could not read frame of video") {
            return Err(Error::VideoReadError("Could not read frame from video".to_string()));
        }

        // Rotate
        if config.should_rotate {
            opencv::core::rotate(&frame.clone(), &mut frame.0, config.rotate).unwrap();
        }

        Ok(frame)
    }
}

impl Reader for OpenCVVideoReader {
    fn total_frames(&self) -> u64 {
        self.total_frames
    }

    fn fps(&self) -> f64 {
        self.fps
    }

    fn read_frame(&mut self, config: &VideoConfig) -> VideoResult<()> {
        let frame = self.read_single_frame(config)?;
        self.frames.push(frame);

        Ok(())
    }

    fn read_frame_as_ascii(&mut self, config: &VideoConfig) -> VideoResult<Vec<Vec<&str>>> {
        let frame = self.read_single_frame(config)?;
        Ok(convert_opencv_video_to_ascii(&frame, &config))
    }

    fn finish(&mut self) -> VideoResult<()> {
        Ok(())
    }
}

pub struct OpenCVVideoWriter {
    pub writer: VideoWriter,
    pub frames: Vec<Mat>,
    input_frames: Vec<UnsafeMat>,
    width: u32,
    height: u32,
    closed: bool,
}
impl OpenCVVideoWriter {
    pub fn new(config: &VideoConfig, reader: OpenCVVideoReader) -> VideoResult<Self> {
        let mut frames = vec![Mat::default(); reader.total_frames as usize];

        let input_frames = reader.frames;
        let ascii = convert_opencv_video_to_ascii(&input_frames[0], &config);
        let (width, height) = get_size_from_ascii(&ascii, config.height_sample_scale, config.font_size);
        // Openh264 codec seems to have this dimension limitation so we cap it
        if width * height > 9437184 {
            // a / b = width / height
            // a * b <= 9437184
            return Err(Error::ResolutionTooLarge);
        }
        frames[0] = encode_ascii_frame_opencv(&config, &ascii, width, height);

        //println!("frame size: {:?}", output_frame_size);
        let video_fps = if config.use_max_fps_for_output_video { config.max_fps as f64 } else { reader.fps };
        // TODO: allow any video output
        let video_writer: VideoWriter = VideoWriter::new(
            config.output_video_path.as_ref().unwrap().as_str(),
            VideoWriter::fourcc('a', 'v', 'c', '1').unwrap(),
            video_fps,
            Size::new(width as i32, height as i32),
            true,
        )
        .unwrap();

        Ok(Self { writer: video_writer, frames, input_frames, width, height, closed: false })
    }
}

impl Encoder for OpenCVVideoWriter {
    fn encode_frame(&mut self, config: &VideoConfig, frame_index: usize) -> VideoResult<()> {
        let ascii = convert_opencv_video_to_ascii(&self.input_frames[frame_index], &config);
        let frame = encode_ascii_frame_opencv(&config, &ascii, self.width, self.height);
        unsafe {
            let ptr = self.frames.as_ptr() as *mut Mat;
            ptr.add(frame_index).write(frame);
        }
        Ok(())
    }
}

impl Writer for OpenCVVideoWriter {
    fn write_frame(&mut self, frame_index: usize) -> VideoResult<()> {
        write_to_ascii_video_opencv(&mut self.writer, &self.frames[frame_index]);
        Ok(())
    }

    fn close(&mut self) -> VideoResult<()> {
        if self.closed {
            Ok(())
        } else {
            self.writer.release().map_err(|e| Error::VideoWriteError(format!("Could not release video writer: {e}")))
        }
    }
}

impl Drop for OpenCVVideoWriter {
    fn drop(&mut self) {
        if let Err(e) = self.close() {
            eprintln!("OpenCVVideoWriter close error: {e}");
        }
    }
}

unsafe impl Sync for OpenCVVideoWriter {}

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
