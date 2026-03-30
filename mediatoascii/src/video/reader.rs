use crate::video::{VideoConfig, VideoResult};

pub trait Reader {
    fn total_frames(&self) -> u64;

    fn fps(&self) -> f64;

    fn read_frame(&mut self, config: &VideoConfig) -> VideoResult<()>;
    fn read_frame_as_ascii(&mut self, config: &VideoConfig) -> VideoResult<Vec<Vec<&str>>>;

    fn finish(&mut self) -> VideoResult<()>;
}
