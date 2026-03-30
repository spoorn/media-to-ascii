use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Resolution too large for codec. Try increasing scale-down setting")]
    ResolutionTooLarge,
    #[error("Failed to read video file: {0}")]
    VideoReadError(String),
    #[error("Failed to write video file: {0}")]
    VideoWriteError(String),
    #[error("Operation cancelled")]
    Cancelled,
}

/// Manually implement Serialize to work with tauri
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
