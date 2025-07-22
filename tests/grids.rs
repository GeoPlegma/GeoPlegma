// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use geo_plegmata::dggrs;

fn test_zones_from_bbox(tool: &str, grid: &str, expected_result: usize) {
    let generator = dggrs::get(tool, grid).expect("Failed to create DGGS adapter");
    let bbox = Some(vec![vec![-77.0, 39.0], vec![-76.0, 40.0]]);
    let result = generator.zones_from_bbox(7, false, bbox.clone()).unwrap();
    assert_eq!(
        !result.zones.len(),
        expected_result,
        "{}: zones_from_bbox returned wrong result",
        grid
    );
}

fn test_zone_from_point(tool: &str, grid: &str, expected_result: usize) {
    let pnt = geo::Point::new(10.9, 4.9);
    let generator = dggrs::get(tool, grid).expect("Failed to create DGGS adapter");
    let result = generator.zone_from_point(6, pnt, false).unwrap();
    println!("{}", result.zones.len());
    assert_eq!(
        result.zones.len(),
        expected_result,
        "{}: zones_from_bbox returned empty result",
        grid
    );
}

macro_rules! zones_from_bbox_test {
    ($name:ident, $tool:expr, $grid:expr, $expected_result:expr) => {
        #[test]
        fn $name() {
            test_zones_from_bbox($tool, $grid, $expected_result);
        }
    };
}

macro_rules! zone_from_point_test {
    ($name:ident, $tool:expr, $grid:expr, $expected_result:expr) => {
        #[test]
        fn $name() {
            test_zone_from_point($tool, $grid, $expected_result);
        }
    };
}

// WARN: Fails because of dggal, run tests with RUST_TEST_THREADS=1 cargo test
zones_from_bbox_test!(
    test_zones_from_bbox_dggal_rtea3h,
    "DGGAL",
    "RTEA3H",
    18446744073709551611
);
zone_from_point_test!(test_zone_from_point_dggal_rtea3h, "DGGAL", "RTEA3H", 1);
