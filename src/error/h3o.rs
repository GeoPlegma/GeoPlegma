use std::num::TryFromIntError;
use thiserror::Error;

/// Error type for zone-related logic in DGGAL-based adapters.
#[derive(Debug, Error)]
pub enum H3oError {
    #[error("Failed to convert edge count to u32 for zone ID '{zone_id}': {source}")]
    EdgeCountConversion {
        zone_id: String,
        #[source]
        source: TryFromIntError,
    },

    #[error("Invalid zone ID format: '{0}'")]
    InvalidZoneIdFormat(String),

    #[error("Invalid H3 zone ID `{0}`: {1}")]
    InvalidZoneID(String, #[source] h3o::error::InvalidCellIndex),

    #[error("Missing required zone data")]
    MissingZoneData,
}
