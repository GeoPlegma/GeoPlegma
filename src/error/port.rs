use crate::error::dggal::DggalError;
use crate::error::dggrid::DggridError;
use crate::error::h3o::H3oError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortError {
    #[error("DGGAL error: {0}")]
    Dggal(#[from] DggalError),

    #[error("DGGRID error: {0}")]
    Dggrid(#[from] DggridError),

    #[error("H3o error: {0}")]
    H3o(#[from] H3oError),

    #[error("Unsupported tool/grid combination: {tool}, {grid}")]
    UnsupportedCombo { tool: String, grid: String },
}
