use geoplegma::api::DggrsApi;
use geoplegma::adapters::h3o::h3::H3Impl;
use geoplegma::models::common::{RefinementLevel, ZoneId};

pub trait Linearizer: DggrsApi {
    fn num_cells_at_level(&self, level: RefinementLevel) -> u64;

    fn zone_to_linear(&self, zone_id: &ZoneId) -> u64;

    fn linear_to_zone(&self, level: RefinementLevel, index: u64) -> ZoneId;
}

impl Linearizer for H3Impl {
    fn num_cells_at_level(&self, level: RefinementLevel) -> u64 {
        let r = level.get();
        if r < 0 {
            return 0;
        }

        // https://h3geo.org/docs/core-library/restable/
        // 122 * 7^r ?
        122_u64.saturating_mul(7_u64.saturating_pow(r as u32))
    }

    fn zone_to_linear(&self, zone_id: &ZoneId) -> u64 {
        match zone_id {
            ZoneId::IntId(v) => *v,
            ZoneId::HexId(h) => u64::from_str_radix(h.as_str(), 16).unwrap_or(0),
            ZoneId::StrId(s) => s.parse::<u64>().unwrap_or(0),
        }
    }

    fn linear_to_zone(&self, _level: RefinementLevel, index: u64) -> ZoneId {
        ZoneId::new_hex(&format!("{index:x}")).unwrap_or(ZoneId::IntId(index))
    }
}
