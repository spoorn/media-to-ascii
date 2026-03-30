use ffmpeg_next::util::frame::video::Video as FfmpegVideoFrame;

#[derive(Clone)]
pub struct FFmpegFrame {
    pub frame: FfmpegVideoFrame,
    pub width: u32,
    pub height: u32,
}

impl FFmpegFrame {
    pub fn new(frame: FfmpegVideoFrame) -> Self {
        let width = frame.width();
        let height = frame.height();
        Self { frame, width, height }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let offset = ((y * self.width + x) * 3) as usize;
        let data = self.frame.data(0);
        (
            data[offset],     // R
            data[offset + 1], // G
            data[offset + 2], // B
        )
    }
}

impl Default for FFmpegFrame {
    fn default() -> Self {
        Self {
            frame: FfmpegVideoFrame::empty(),
            width: 0,
            height: 0,
        }
    }
}
