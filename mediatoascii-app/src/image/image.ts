export interface ImageConfig {
  image_path: string;
  scale_down: number;
  font_size: number;
  height_sample_scale: number;
  invert: boolean;
  output_file_path?: string | null;
  output_image_path?: string | null;
  overwrite: boolean;
}

export function defaultImageConfig(): ImageConfig {
  return {
    image_path: "",
    scale_down: 1.0,
    font_size: 12.0,
    height_sample_scale: 2.046,
    invert: false,
    output_file_path: null,
    output_image_path: null,
    overwrite: false,
  };
}
