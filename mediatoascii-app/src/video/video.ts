export interface VideoConfig {
    video_path: string;
    scale_down: number;
    font_size: number;
    height_sample_scale: number;
    invert: boolean;
    max_fps: number;
    output_video_path: string | null;
    overwrite: boolean;
    use_max_fps_for_output_video: boolean;
    rotate: number;
}

export function defaultVideoConfig(): VideoConfig {
    return {
        video_path: "",
        output_video_path: null,
        scale_down: 1.0,
        font_size: 12.0,
        height_sample_scale: 2.046,
        invert: false,
        max_fps: 10,
        overwrite: true, // Always true - rely on dialog box warning when file exists
        use_max_fps_for_output_video: false,
        rotate: -1,
    };
}

export interface RotateOption {
    label: string;
    value: number;
}

export const rotateOptions: RotateOption[] = [
    { label: "No Rotation", value: -1 },
    { label: "90° Clockwise", value: 0 },
    { label: "180° Flip", value: 1 },
    { label: "90° Counter-Clockwise", value: 2 },
];
