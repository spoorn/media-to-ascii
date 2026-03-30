use crate::video::{VideoConfig, VideoResult};

pub trait Encoder {
    fn encode_frame(&mut self, config: &VideoConfig, frame_index: usize) -> VideoResult<()>;
}
