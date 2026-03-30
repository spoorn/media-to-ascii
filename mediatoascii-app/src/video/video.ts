export interface VideoConfig {
    video_path: string;
    scale_down: number;
    font_size: number;
    height_sample_scale: number;
    invert: boolean;
    max_fps: number;
    bitrate: number;
    output_video_path: string | null;
    overwrite: boolean;
    use_max_fps_for_output_video: boolean;
    rotate: number;
    //num_threads: number;
}

export function defaultVideoConfig(): VideoConfig {
    return {
        video_path: "",
        output_video_path: null,
        scale_down: 4.0,
        font_size: 12.0,
        height_sample_scale: 2.046,
        invert: false,
        max_fps: 10,
        bitrate: 4000000,
        overwrite: true, // Always true - rely on dialog box warning when file exists
        use_max_fps_for_output_video: false,
        rotate: -1,
        //num_threads: getNumThreads(),
    };
}

export function getNumThreads() {
    let threads = 4; // fallback
    if (typeof navigator !== "undefined") {
        // Running in browser/webview
        threads = navigator.hardwareConcurrency || 4;
    }
    return threads;
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
