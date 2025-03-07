#!/usr/bin/env python3
"""
Example script demonstrating how to use the mediatoascii Python bindings.
"""
import os
import argparse
import mediatoascii

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
    
    args = parser.parse_args()
    
    if not args.image and not args.video:
        parser.error("Either --image or --video must be specified")
    
    if args.image and args.video:
        parser.error("Only one of --image or --video can be specified")
    
    print(f"Using mediatoascii version: {mediatoascii.__version__}")
    
    if args.image:
        print(f"Converting image: {args.image}")
        result = mediatoascii.image_to_ascii(
            args.image,
            scale_down=args.scale,
            font_size=args.font_size,
            invert=args.invert,
            output_file_path=args.output
        )
        print(result)
    
    if args.video:
        print(f"Converting video: {args.video}")
        result = mediatoascii.video_to_ascii(
            args.video,
            scale_down=args.scale,
            font_size=args.font_size,
            invert=args.invert,
            max_fps=args.max_fps,
            output_video_path=args.output,
            use_max_fps_for_output_video=args.use_max_fps
        )
        print(result)

if __name__ == "__main__":
    main() 