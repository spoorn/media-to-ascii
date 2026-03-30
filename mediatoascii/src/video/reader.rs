use crate::video::VideoResult;

pub trait Reader {
    fn total_frames(&self) -> u64;

    fn fps(&self) -> f64;

    fn read_frame(&mut self, should_rotate: bool, rotate: i32) -> VideoResult<()>;

    fn finish(&mut self) -> VideoResult<()>;
}