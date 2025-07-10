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
use geo_plegmata::models::common::Zones; // Adjust path as needed

#[test]
fn test_all_grids() {
    let configs = vec![
        ("DGGAL", "IVEA3H", "1297036692682743552"),
        ("DGGAL", "IVEA9R", "1297036692682743552"),
        ("DGGAL", "ISEA3H", "1297036692682743552"),
        ("DGGAL", "ISEA9R", "1297036692682743552"),
        ("DGGAL", "RTEA3H", "1297036692682743552"),
        ("DGGAL", "RTEA9R", "1297036692682743552"),
        ("H3O", "H3", "811fbffffffffff"),
    ];

    let bbox: Option<Vec<Vec<f64>>> = Some(vec![
        vec![-77.0, 39.0], // lower left
        vec![-76.0, 40.0], // upper right
    ]);

    let pnt = Point::new(10.9, 4.9);

    for (tool, dggs, zone_id) in configs {
        let generator = dggrs::get(tool, dggs).expect("Factory failed to create DGGS adapter");

        // Global
        let r: Zones = generator
            .zones_from_bbox(2, false, None)
            .expect("zone generation failed");
        assert!(r.zones.len() == 3);

        // // Global with bbox
        // let r: Zones = generator
        //     .zones_from_bbox(7, false, bbox.clone())
        //     .expect("zone generation failed");
        // assert!(r.is_ok());

        // // From point
        // let r: Zones = generator
        //     .zone_from_point(6, pnt, false)
        //     .expect("zone generation failed");
        // assert!(r.is_ok());

        // // From parent
        // let r: Zones = generator
        //     .zones_from_parent(6, zone_id.to_string(), false)
        //     .expect("zone generation failed");
        // assert!(r.is_ok());

        // // From ID
        // let r: Zones = generator
        //     .zone_from_id(zone_id.to_string(), false)
        //     .expect("zone generation failed");
        // assert!(r.is_ok());
    }
}
