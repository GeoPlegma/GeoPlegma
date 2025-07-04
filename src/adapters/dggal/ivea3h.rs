// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::dggal::dggal::DggalAdapter;
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use geo::{LineString, Point, Polygon};
extern crate ecrt;
use ecrt::{Application, tokenizeWith};
extern crate dggal;
use crate::adapters::dggal::common::{ids_to_zones, to_geo_extent, to_geo_point};
use dggal::{DGGAL, DGGRS, GeoExtent, GeoPoint, wholeWorld};
use std::env;

pub struct Ivea3hImpl {
    pub adapter: DggalAdapter,
}

impl Ivea3hImpl {
    pub fn new() -> Self {
        Self {
            adapter: DggalAdapter::new(),
        }
    }
}

impl Default for Ivea3hImpl {
    fn default() -> Self {
        Self {
            adapter: DggalAdapter::default(),
        }
    }
}

fn get_dggrs() -> DGGRS {
    let args: Vec<String> = env::args().collect();
    let my_app = Application::new(&args);
    let dggal = DGGAL::new(&my_app);
    let dggrs: DGGRS = DGGRS::new(&dggal, "IVEA3H").expect("Unknown DGGRS");
    dggrs
}

impl DggrsPort for Ivea3hImpl {
    fn zones_from_bbox(&self, depth: u8, densify: bool, bbox: Option<Vec<Vec<f64>>>) -> Zones {
        let d = get_dggrs();
        let max_depth = d.getMaxDepth();

        let capped_depth = if depth as i32 > max_depth {
            max_depth
        } else {
            depth as i32
        };

        let zones = if let Some(b) = bbox {
            d.listZones(capped_depth, &to_geo_extent(Some(b)))
        } else {
            d.listZones(capped_depth, &wholeWorld)
        };

        ids_to_zones(d, zones)
    }
    fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Zones {
        let d = get_dggrs();
        let zone = d.getZoneFromWGS84Centroid(8, &to_geo_point(point));
        let zones = vec![zone];
        ids_to_zones(d, zones)
    }
    fn zones_from_parent(&self, depth: u8, parent_zone_id: String, densify: bool) -> Zones {
        let d = get_dggrs();
        let max_depth = d.getMaxDepth();

        let capped_depth = if depth as i32 > max_depth {
            max_depth
        } else {
            depth as i32
        };

        let num: u64 = parent_zone_id.parse::<u64>().expect("Invalid u64 string"); // FIX: parent_zone_id needs to be the ZoneID enum not String
        let zones = d.getSubZones(num, 5);

        ids_to_zones(d, zones)
    }
    fn zone_from_id(&self, zone_id: String, densify: bool) -> Zones {
        let d = get_dggrs();
        let num: u64 = zone_id.parse::<u64>().expect("Invalid u64 string"); // FIX: parent_zone_id needs to be the ZoneID enum not String
        let zones = vec![num];
        ids_to_zones(d, zones)
    }
}
