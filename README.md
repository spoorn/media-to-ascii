# Media-To-Ascii
CLI and utilities for converting media files (images/videos) to ascii outputs (output media file or print to console).  
Supports most standard image formats, and some video formats.

Only output format for videos is `.mp4` at the moment

# How To Use

## Basic Usage

### Videos

```commandline
# Playing videos as ascii art in the console (warning, for large videos, this may cause flickering on certain terminals)
mediatoascii --video-path <FILE_PATH>

# Saving ascii art as a video file (only .mp4 is supported as the output format)
mediatoascii --video-path <FILE_PATH> -o ascii.mp4
```

### Images
```commandline
# Converting images to ascii in the console
mediatoascii --image-path <IMAGE_PATH>

# Outputting ascii images in an image file
mediatoascii --image-path <FILE_PATH> -o ascii.png

# Outputting ascii images as ascii text in a file
mediatoascii --image-path <FILE_PATH> -o ascii.txt --as-text
```

### For the full set of features, see the `--help` menu:

```commandline
$ mediatoascii --help
mediatoascii 0.1.0
spoorn
CLI and utilities for converting media files (images/videos) to ascii outputs (output media file or
print to console).
Supports most standard image formats, and video formats.

USAGE:
    mediatoascii [OPTIONS] <--image-path <IMAGE_PATH>|--video-path <VIDEO_PATH>>

OPTIONS:
        --image-path <IMAGE_PATH>
            Input Image file.  One of image_path, or video_path must be populated

        --video-path <VIDEO_PATH>
            Input Video file.  One of image_path, or video_path must be populated

        --scale-down <SCALE_DOWN>
            Multiplier to scale down input dimensions by when converting to ASCII.  For large
            frames, recommended to scale down more so output file size is more reasonable [default:
            1]

        --height-sample-scale <HEIGHT_SAMPLE_SCALE>
            Rate at which we sample from the pixel rows of the frames.  This affects how stretched
            the output ascii is in the vertical or y-axis [default: 2.4]

    -i, --invert
            Invert ascii greyscale ramp (For light backgrounds.  Default OFF is for dark
            backgrounds.)

        --overwrite
            Overwrite any output file if it already exists

        --max-fps <MAX_FPS>
            Max FPS for video outputs.  If outputting to video file, `use_max_fps_for_output_video`
            must be set to `true` to honor this setting.  Ascii videos in the terminal default to
            max_fps=10 for smoother visuals

        --as-text
            For images, if output_file_path is specified, will save the ascii text as-is to the
            output rather than an image file

    -o, --output-file-path <OUTPUT_FILE_PATH>
            Output file path.  If omitted, output will be written to console. Supports most image
            formats, and .mp4 video outputs

        --use-max-fps-for-output-video
            Use the max_fps setting for video file outputs

    -r, --rotate <ROTATE>
            Rotate the input (0 = 90 CLOCKWISE, 1 = 180, 2 = 90 COUNTER-CLOCKWISE)

    -h, --help
            Print help information

    -V, --version
            Print version information
```

# Installation

## Crates.io

```commandline
cargo install mediatoascii
```

## Git

```commandline
# Clone repository and cd into it
git clone ...
cd mediatoascii/

# Install the package into cargo
cargo install --path .

# OR run the pre-packaged binary in bin/
./bin/mediatoascii ...

# OR via `cargo run`
cargo run --release
```

# Development

## For Videos
Make sure you have OpenCV installed if running via `cargo run` or from the source: https://github.com/twistedfall/opencv-rust
