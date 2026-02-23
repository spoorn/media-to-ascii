# Live Camera Feed Feature - Implementation Summary

## Overview
Successfully implemented live camera feed support for the media-to-ascii CLI tool. The implementation allows real-time ASCII art conversion from any connected camera device.

## What Changed

### 1. Core Library (`mediatoascii/src/video/video.rs`)
- **Added `camera_index` field** to `VideoConfig` struct
- **Modified `process_video()` function** to support both file and camera modes:
  - Detects camera mode via `camera_index.is_some()`
  - Uses `VideoCapture::new(camera_idx, CAP_ANY)` for camera input
  - Implements infinite loop for live feed (exits on Ctrl+C or read error)
  - Improved progress reporting (frame counter for camera mode)
  - Better terminal output with flush for smoother display

### 2. CLI (`mediatoascii-cli/src/main.rs`)
- **Added `--camera-index` argument**
- **Updated argument group** to include camera as a valid input option
- Camera index, video path, and image path are mutually exclusive

## Usage Examples

### Basic Camera Feed
```bash
# Terminal output (default camera)
./target/release/mediatoascii-cli --camera-index 0

# Use second camera
./target/release/mediatoascii-cli --camera-index 1
```

### Performance Tuning
```bash
# High performance mode (scale down 4x, 30 FPS)
./target/release/mediatoascii-cli --camera-index 0 --scale-down 4.0 --max-fps 30

# Lower resolution for better FPS
./target/release/mediatoascii-cli --camera-index 0 --scale-down 2.0 --font-size 8.0 --max-fps 20

# Maximum quality (slower FPS)
./target/release/mediatoascii-cli --camera-index 0 --scale-down 1.0 --font-size 16.0 --max-fps 10
```

### Recording Camera Feed
```bash
# Record to video file
./target/release/mediatoascii-cli --camera-index 0 -o camera_output.mp4

# Record with custom FPS
./target/release/mediatoascii-cli --camera-index 0 -o output.mp4 --use-max-fps-for-output-video --max-fps 24
```

### Advanced Options
```bash
# Inverted colors (for light terminal backgrounds)
./target/release/mediatoascii-cli --camera-index 0 --invert

# Rotated 90° clockwise
./target/release/mediatoascii-cli --camera-index 0 --rotate 0

# Combined: scaled, inverted, with custom font size
./target/release/mediatoascii-cli --camera-index 0 --scale-down 2.5 --font-size 10.0 --invert --max-fps 25
```

## Performance Expectations

Based on the implementation:

| Resolution | scale_down | Expected FPS | Quality |
|------------|-----------|--------------|---------|
| 640x480 | 4.0 | 30-60 | Low, very fast |
| 640x480 | 2.0 | 20-30 | Medium |
| 640x480 | 1.0 | 10-15 | High quality |
| 1280x720 | 4.0 | 25-40 | Medium |
| 1280x720 | 2.0 | 15-25 | High |
| 1920x1080 | 6.0 | 20-30 | Medium |

## Building the Project

**Important:** You need to set environment variables for the build to work:

```bash
# Required environment variables (add to ~/.zshrc for permanent)
export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib
export DYLD_LIBRARY_PATH=/opt/homebrew/opt/llvm/lib:$DYLD_LIBRARY_PATH

# Build
cargo build -p mediatoascii-cli --release

# The binary will be at:
./target/release/mediatoascii-cli
```

### Prerequisites
If you haven't already, install:
```bash
brew install pkgconf opencv llvm
```

## Future Enhancements (Not Yet Implemented)

### Potential Tauri UI Features
Your project already has a Tauri desktop app (`mediatoascii-app`). Future enhancements could include:
- Camera device selection dropdown
- Live preview window with ASCII rendering
- Start/stop controls with recording
- Real-time FPS counter
- Settings panel for all parameters
- Snapshot capture from live feed

### Additional CLI Features
- Multi-camera support (split screen)
- Effects/filters during capture
- Motion detection triggers
- Time-lapse recording
- Configurable keyboard shortcuts

## Technical Details

### Camera Initialization
```rust
let mut capture = if let Some(camera_idx) = config.camera_index {
    videoio::VideoCapture::new(camera_idx, CAP_ANY)
        .unwrap_or_else(|_| panic!("Could not open camera device {}", camera_idx))
} else {
    // File mode
    videoio::VideoCapture::from_file(video_path, CAP_ANY)
        .unwrap_or_else(|_| panic!("Could not open video file at {video_path}"))
};
```

### Infinite Loop for Live Feed
```rust
let num_frames = if is_camera_mode {
    u64::MAX  // Infinite for camera mode
} else {
    capture.get(videoio::CAP_PROP_FRAME_COUNT).unwrap() as u64
};
```

### Frame Rate Control
The implementation uses the `max_fps` config to control display rate:
```rust
let target_frame_time = 1.0 / config.max_fps as f64;
if elapsed < target_frame_time {
    sleep(Duration::from_millis(((target_frame_time - elapsed) * 1000.0) as u64));
}
```

## Known Limitations

1. **Terminal flicker**: For large outputs, terminal rendering may flicker (mentioned in original README)
   - Solution: Use video file output instead (`-o output.mp4`)

2. **Camera availability**: Camera must not be in use by another application

3. **Resolution limits**: Video encoding has max resolution of ~9.4MP
   - Error: `ResolutionTooLarge`
   - Solution: Increase `--scale-down` parameter

## Testing

To test camera functionality:
```bash
# Quick test (should show live feed)
./target/release/mediatoascii-cli --camera-index 0 --scale-down 4.0 --max-fps 30

# Test recording
./target/release/mediatoascii-cli --camera-index 0 --scale-down 2.0 -o test_camera.mp4

# Verify help shows camera option
./target/release/mediatoascii-cli --help | grep camera
```

## Troubleshooting

### "Could not open camera device 0"
- Ensure camera is not in use by another app
- Try different indices (0, 1, 2)
- Check camera permissions (macOS: System Settings → Privacy & Security → Camera)

### Build fails with libclang error
```bash
# Set these environment variables
export LIBCLANG_PATH=/opt/homebrew/opt/llvm/lib
export DYLD_LIBRARY_PATH=/opt/homebrew/opt/llvm/lib:$DYLD_LIBRARY_PATH
```

### Poor performance
- Increase `--scale-down` (try 3.0 or 4.0)
- Reduce `--font-size` (try 8.0)
- Lower `--max-fps` (try 15 or 20)

---

**Status**: ✅ CLI Implementation Complete
**Next Steps**: Consider adding Tauri UI integration for better UX
