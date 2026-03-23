use geoplegma::models::common::ZoneId;

use crate::error::EncodingError;

pub(crate) fn zone_id_to_u64(id: &ZoneId) -> Result<u64, EncodingError> {
    match id {
        ZoneId::IntId(v) => Ok(*v),
        ZoneId::HexId(h) => u64::from_str_radix(h.as_str(), 16)
            .map_err(|e| EncodingError::Grid(format!("invalid hex zone id {h}: {e}"))),
        ZoneId::StrId(s) => s
            .parse::<u64>()
            .map_err(|e| EncodingError::Grid(format!("invalid string zone id {s}: {e}"))),
    }
}