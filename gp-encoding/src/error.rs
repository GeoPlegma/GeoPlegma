use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GDAL error: {0}")]
    Gdal(#[from] gdal::errors::GdalError),

    #[error("DGGRS error: {0}")]
    Dggrs(#[from] geoplegma::error::DggrsError),

    #[error("DGGRS Fabric error: {0}")]
    DggrsFabric(#[from] geoplegma::error::factory::FactoryError),

    #[error("GeoTIFF error: {0}")]
    GeoTiff(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Zarr backend error: {0}")]
    Zarr(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Grid error: {0}")]
    Grid(String),
}
