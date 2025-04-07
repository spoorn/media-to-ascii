export interface VideoConfig {
    video_path: string;
    scale_down: number;
    font_size: number;
    height_sample_scale: number;
    invert: boolean;
    max_fps: number;
    output_video_path: string;
    overwrite: boolean;
    use_max_fps_for_output_video: boolean;
    rotate: number;
}

export function defaultVideoConfig(): VideoConfig {
    return {
        video_path: "",
        scale_down: 1.0,
        font_size: 12.0,
        height_sample_scale: 2.046,
        invert: false,
        max_fps: 60,
        output_video_path: "",
        overwrite: false,
        use_max_fps_for_output_video: false,
        rotate: 0,
    };
}
