#!/usr/bin/env python3
"""
Example script demonstrating how to use the mediatoascii Python bindings.
"""
import os
import argparse
import mediatoascii

def process_image_bytes(image_path, scale_down, font_size, invert, output_path=None, custom_chars=None):
    """Process image using the bytes-based method."""
    print(f"Converting image (bytes method): {image_path}")
    
    # Read the image file into bytes
    with open(image_path, "rb") as f:
        image_bytes = f.read()
    
    # Convert bytes to ASCII
    ascii_art = mediatoascii.image_bytes_to_ascii(
        image_bytes,
        scale_down=scale_down,
        font_size=font_size,
        invert=invert,
        custom_chars=custom_chars.split() if custom_chars else None
    )
    
    # Join the ASCII art into a single string with \r\n line endings
    ascii_str = '\r\n'.join(''.join(row) for row in ascii_art) + '\r\n'
    
    # Write to file if output path is specified
    if output_path:
        with open(output_path, 'w', newline='') as f:  # Use newline='' to prevent extra conversions
            f.write(ascii_str)
        print(f"ASCII art saved to: {output_path}")
    else:
        print(ascii_str)

def process_image_file(image_path, scale_down, font_size, invert, output_path=None, custom_chars=None):
    """Process image using the file-based method."""
    print(f"Converting image (file method): {image_path}")
    result = mediatoascii.image_to_ascii(
        image_path,
        scale_down=scale_down,
        font_size=font_size,
        invert=invert,
        output_file_path=output_path,
        custom_chars=custom_chars.split() if custom_chars else None
    )
    print(result)

def main():
    parser = argparse.ArgumentParser(description="Convert media to ASCII art")
    parser.add_argument("--image", type=str, help="Path to input image file")
    parser.add_argument("--video", type=str, help="Path to input video file")
    parser.add_argument("--output", type=str, help="Path to output file")
    parser.add_argument("--scale", type=float, default=1.0, help="Scale down factor")
    parser.add_argument("--font-size", type=float, default=12.0, help="Font size")
    parser.add_argument("--invert", action="store_true", help="Invert ASCII greyscale ramp")
    parser.add_argument("--max-fps", type=int, help="Maximum FPS for video outputs")
    parser.add_argument("--use-max-fps", action="store_true", help="Use max FPS for video file outputs")
    parser.add_argument("--use-bytes", action="store_true", help="Use bytes-based processing method")
    parser.add_argument("--chars", type=str, help="Custom character set (space-separated, from darkest to lightest)")
    
    args = parser.parse_args()
    
    if not args.image and not args.video:
        parser.error("Either --image or --video must be specified")
    
    if args.image and args.video:
        parser.error("Only one of --image or --video can be specified")
    
    print(f"Using mediatoascii version: {mediatoascii.__version__}")
    
    if args.image:
        if args.use_bytes:
            process_image_bytes(
                args.image,
                scale_down=args.scale,
                font_size=args.font_size,
                invert=args.invert,
                output_path=args.output,
                custom_chars=args.chars
            )
        else:
            process_image_file(
                args.image,
                scale_down=args.scale,
                font_size=args.font_size,
                invert=args.invert,
                output_path=args.output,
                custom_chars=args.chars
            )
    
    if args.video:
        print(f"Converting video: {args.video}")
        result = mediatoascii.video_to_ascii(
            args.video,
            scale_down=args.scale,
            font_size=args.font_size,
            invert=args.invert,
            max_fps=args.max_fps,
            output_video_path=args.output,
            use_max_fps_for_output_video=args.use_max_fps,
            custom_chars=args.chars.split() if args.chars else None
        )
        print(result)

if __name__ == "__main__":
    main() 