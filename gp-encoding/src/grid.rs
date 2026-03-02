use api::api::DggrsApi;
use api::models::common::{RefinementLevel, ZoneId};

pub trait Linearizer: DggrsApi {
    fn num_cells_at_level(&self, level: RefinementLevel) -> u64;

    fn zone_to_linear(&self, zone_id: &ZoneId) -> u64;

    fn linear_to_zone(&self, level: RefinementLevel, index: u64) -> ZoneId;
}
