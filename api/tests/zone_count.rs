use api::adapters::dggal::grids::DggalImpl;
use api::adapters::dggrid::igeo7::Igeo7Impl;
use api::adapters::dggrid::isea3h::Isea3hImpl;
use api::adapters::h3o::h3::H3Impl;
use api::api::DggrsApi;
use api::models::common::RefinementLevel;

/// Verify that zone_count matches zones_from_bbox count with an empty bbox (entire globe)
fn test_adapter_zone_count_equivalence<T: DggrsApi>(adapter: &T) {
    for level_int in 0..=2 {
        let level = RefinementLevel::new(level_int).unwrap();

        let num_zones = adapter.zone_count(level).unwrap();
        let zones = adapter.zones_from_bbox(level, None, None).unwrap();
        let zones_count = zones.zones.len() as u64;

        assert_eq!(num_zones, zones_count);
    }
}

#[test]
fn test_isea3h_dggrid_dggal_zone_count_equivalence() {
    let dggal = DggalImpl::new(api::models::common::DggrsUid::ISEA3HDGGAL);
    let dggrid = Isea3hImpl::default();

    let max_level = dggal.max_refinement_level().unwrap();

    for level_int in 0..=max_level.get() {
        let level = RefinementLevel::new(level_int).unwrap();

        let dggal_zone_count = dggal.zone_count(level).unwrap();
        let dggrid_zone_count = dggrid.zone_count(level).unwrap();

        assert_eq!(dggal_zone_count, dggrid_zone_count);
    }
}

#[test]
fn test_h3_zone_count_equivalence() {
    let h3 = H3Impl::new();
    test_adapter_zone_count_equivalence(&h3);
}

#[test]
fn test_dggal_zone_count_equivalence() {
    let dggal = DggalImpl::new(api::models::common::DggrsUid::ISEA3HDGGAL);
    test_adapter_zone_count_equivalence(&dggal);
}

#[test]
fn test_igeo7_zone_count_equivalence() {
    let igeo7 = Igeo7Impl::default();
    test_adapter_zone_count_equivalence(&igeo7);
}

#[test]
fn test_isea3h_zone_count_equivalence() {
    let isea3h = Isea3hImpl::default();
    test_adapter_zone_count_equivalence(&isea3h);
}
