use h3o::error::InvalidCellIndex;
use h3o::error::InvalidResolution;
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

    #[error("Depth {depth} for {zone_id} is to high: {source}")]
    InvalidResolution {
        zone_id: String,
        depth: u8,
        #[source]
        source: InvalidResolution, // TODO: is this the right error from the h3o crate?
    },

    #[error("Invalid H3 zone ID {zone_id}: {source}")]
    InvalidZoneID {
        zone_id: String,
        #[source]
        source: InvalidCellIndex,
    },

    #[error("Invalid resolution for H3 zone ID {zone_id}")]
    ResolutionLimitReached { zone_id: String },

    #[error("Missing required zone data")]
    MissingZoneData,
}
