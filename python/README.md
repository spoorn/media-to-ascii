# Media to ASCII Python Bindings

Python bindings for the [media-to-ascii](https://github.com/spoorn/media-to-ascii) Rust library, which converts images and videos to ASCII art.

## Installation

```bash
pip install mediatoascii
```

## Usage

### Converting an Image to ASCII

```python
import mediatoascii

# Basic usage - prints to console
mediatoascii.image_to_ascii("path/to/image.jpg")

# Save as text file
mediatoascii.image_to_ascii(
    "path/to/image.jpg",
    as_text=True,
    output_file_path="output.txt"
)

# Save as image file with custom settings
mediatoascii.image_to_ascii(
    "path/to/image.jpg",
    scale_down=2.0,
    font_size=14.0,
    invert=True,
    output_file_path="output.png"
)
```

### Converting a Video to ASCII

```python
import mediatoascii

# Basic usage - prints to console
mediatoascii.video_to_ascii("path/to/video.mp4")

# Save as video file with custom settings
mediatoascii.video_to_ascii(
    "path/to/video.mp4",
    scale_down=2.0,
    font_size=14.0,
    max_fps=30,
    output_file_path="output.mp4",
    use_max_fps_for_output_video=True
)
```

## Parameters

### Image to ASCII

- `image_path`: Path to the input image file
- `scale_down`: Multiplier to scale down input dimensions (default: 1.0)
- `font_size`: Font size of the ASCII characters (default: 12.0)
- `height_sample_scale`: Rate at which to sample from pixel rows (default: 2.046)
- `invert`: Invert ASCII greyscale ramp for light backgrounds (default: False)
- `as_text`: Save as text file instead of image (default: False)
- `output_file_path`: Path to save output (if None, prints to console)
- `rotate`: Rotate the input (0 = 90° clockwise, 1 = 180°, 2 = 90° counter-clockwise)

### Video to ASCII

- `video_path`: Path to the input video file
- `scale_down`: Multiplier to scale down input dimensions (default: 1.0)
- `font_size`: Font size of the ASCII characters (default: 12.0)
- `height_sample_scale`: Rate at which to sample from pixel rows (default: 2.046)
- `invert`: Invert ASCII greyscale ramp for light backgrounds (default: False)
- `overwrite`: Overwrite output file if it exists (default: False)
- `max_fps`: Maximum FPS for video outputs (default: None)
- `output_file_path`: Path to save output (if None, prints to console)
- `use_max_fps_for_output_video`: Use max_fps for video file outputs (default: False)
- `rotate`: Rotate the input (0 = 90° clockwise, 1 = 180°, 2 = 90° counter-clockwise) 