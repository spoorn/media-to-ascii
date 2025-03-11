use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::PyDict;

use crate::util::constants::MAGIC_HEIGHT_TO_WIDTH_RATIO;

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
    custom_chars: Option<Vec<String>>,
    preserve_color: Option<bool>,
) -> PyResult<PyObject> {
    // Create a builder with default values
    let mut builder = image::ImageConfigBuilder::default();
    
    // Set required fields
    builder.image_path(image_path.clone());
    
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

    if let Some(val) = custom_chars {
        builder.custom_chars(Some(val));
    }

    if let Some(val) = preserve_color {
        builder.preserve_color(val);
    }
    
    // Build the config
    let config = builder.build().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to build image config: {}", e))
    })?;
    
    // Allow other Python threads to run during the potentially long-running operation
    let output = py.allow_threads(|| {
        // First get the ASCII/color data
        let output = image::convert_image_to_ascii(&config);
        
        // Then handle any file output
        image::process_image(config);
        
        output
    });

    // Convert output to Python dictionary
    let result = PyDict::new(py);
    result.set_item("ascii", output.ascii)?;
    if let Some(colors) = output.colors {
        result.set_item("colors", colors)?;
    }
    Ok(result.into())
}

/// Convert image bytes to ASCII art
#[pyfunction]
fn image_bytes_to_ascii(
    py: Python<'_>,
    image_bytes: &[u8],
    scale_down: Option<f32>,
    font_size: Option<f32>,
    height_sample_scale: Option<f32>,
    invert: Option<bool>,
    custom_chars: Option<Vec<String>>,
    preserve_color: Option<bool>,
) -> PyResult<PyObject> {
    // Create a builder with default values
    let mut builder = image::ImageConfigBuilder::default();
    
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

    if let Some(val) = custom_chars {
        builder.custom_chars(Some(val));
    }

    if let Some(val) = preserve_color {
        builder.preserve_color(val);
    }
    
    // Build the config
    let config = builder.build().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to build image config: {}", e))
    })?;
    
    // Allow other Python threads to run during the potentially long-running operation
    let output = py.allow_threads(|| {
        // Convert bytes to ASCII
        image::convert_image_bytes_to_ascii(image_bytes, &config)
    });
    
    // Convert output to Python dictionary
    let result = PyDict::new(py);
    result.set_item("ascii", output.ascii)?;
    if let Some(colors) = output.colors {
        result.set_item("colors", colors)?;
    }
    Ok(result.into())
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
    custom_chars: Option<Vec<String>>,
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

    if let Some(val) = custom_chars {
        builder.custom_chars(Some(val));
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

/// Write ASCII art to PNG file
#[pyfunction]
fn write_ascii_to_png(
    py: Python<'_>,
    ascii_art: Vec<Vec<String>>,
    output_path: String,
    height_sample_scale: Option<f32>,
    font_size: Option<f32>,
    invert: Option<bool>,
    overwrite: Option<bool>,
) -> PyResult<String> {
    // Set default values if not provided
    let height_sample_scale = height_sample_scale.unwrap_or(MAGIC_HEIGHT_TO_WIDTH_RATIO);
    let font_size = font_size.unwrap_or(12.0);
    let invert = invert.unwrap_or(false);
    let overwrite = overwrite.unwrap_or(false);
    
    // Allow other Python threads to run during the potentially long-running operation
    py.allow_threads(|| {
        image::write_ascii_to_png(&ascii_art, &output_path, height_sample_scale, font_size, invert, overwrite)
    });
    
    Ok("ASCII art PNG saved successfully".to_string())
}

/// Python module for media-to-ascii
#[pymodule]
fn mediatoascii(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(image_to_ascii, m)?)?;
    m.add_function(wrap_pyfunction!(image_bytes_to_ascii, m)?)?;
    m.add_function(wrap_pyfunction!(video_to_ascii, m)?)?;
    m.add_function(wrap_pyfunction!(write_ascii_to_png, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
} 