use crate::video::VideoResult;

pub trait Writer {
    fn write_frame(&mut self, frame_index: usize) -> VideoResult<()>;

    fn close(&mut self) -> VideoResult<()>;
}
