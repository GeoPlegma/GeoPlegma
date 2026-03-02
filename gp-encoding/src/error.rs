use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zarr backend error: {0}")]
    Zarr(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Grid error: {0}")]
    Grid(String),
}
