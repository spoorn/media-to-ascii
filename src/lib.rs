use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod image;
mod util;
mod video;

/// Convert an image to ASCII art
#[pyfunction]
fn image_to_ascii(
    py: Python<'_>,
    image_path: String,
    scale_down: Option<f32>,
    font_size: Option<f32>,
    height_sample_scale: Option<f32>,
    invert: Option<bool>,
    output_file_path: Option<String>,
    output_image_path: Option<String>,
    overwrite: Option<bool>,
) -> PyResult<String> {
    // Create a builder with default values
    let mut builder = image::ImageConfigBuilder::default();
    
    // Set required fields
    builder.image_path(image_path);
    
    // Set optional fields if provided
    if let Some(val) = scale_down {
        builder.scale_down(val);
    }
    
    if let Some(val) = font_size {
        builder.font_size(val);
    }
    
    if let Some(val) = height_sample_scale {
        builder.height_sample_scale(val);
    }
    
    if let Some(val) = invert {
        builder.invert(val);
    }
    
    if let Some(val) = output_file_path {
        builder.output_file_path(Some(val));
    }
    
    if let Some(val) = output_image_path {
        builder.output_image_path(Some(val));
    }
    
    if let Some(val) = overwrite {
        builder.overwrite(val);
    }
    
    // Build the config
    let config = builder.build().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to build image config: {}", e))
    })?;
    
    // Allow other Python threads to run during the potentially long-running operation
    py.allow_threads(|| {
        // Process the image
        image::process_image(config);
    });
    
    Ok("Image processed successfully".to_string())
}

/// Convert a video to ASCII art
#[pyfunction]
fn video_to_ascii(
    py: Python<'_>,
    video_path: String,
    scale_down: Option<f32>,
    font_size: Option<f32>,
    height_sample_scale: Option<f32>,
    invert: Option<bool>,
    overwrite: Option<bool>,
    max_fps: Option<u64>,
    output_video_path: Option<String>,
    use_max_fps_for_output_video: Option<bool>,
    rotate: Option<i32>,
) -> PyResult<String> {
    // Create a builder with default values
    let mut builder = video::VideoConfigBuilder::default();
    
    // Set required fields
    builder.video_path(video_path);
    
    // Set optional fields if provided
    if let Some(val) = scale_down {
        builder.scale_down(val);
    }
    
    if let Some(val) = font_size {
        builder.font_size(val);
    }
    
    if let Some(val) = height_sample_scale {
        builder.height_sample_scale(val);
    }
    
    if let Some(val) = invert {
        builder.invert(val);
    }
    
    if let Some(val) = overwrite {
        builder.overwrite(val);
    }
    
    if let Some(val) = max_fps {
        builder.max_fps(val);
    }
    
    if let Some(val) = output_video_path {
        builder.output_video_path(Some(val));
    }
    
    if let Some(val) = use_max_fps_for_output_video {
        builder.use_max_fps_for_output_video(val);
    }
    
    if let Some(val) = rotate {
        builder.rotate(val);
    }
    
    // Build the config
    let config = builder.build().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to build video config: {}", e))
    })?;
    
    // Allow other Python threads to run during the potentially long-running operation
    py.allow_threads(|| {
        // Process the video
        video::process_video(config);
    });
    
    Ok("Video processed successfully".to_string())
}

/// Python module for media-to-ascii
#[pymodule]
fn mediatoascii(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(image_to_ascii, m)?)?;
    m.add_function(wrap_pyfunction!(video_to_ascii, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
} 