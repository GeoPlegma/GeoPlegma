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
use geo::Point;
extern crate ecrt;
use ecrt::Application;
extern crate dggal;
use crate::adapters::dggal::common::{ids_to_zones, to_geo_extent, to_geo_point};
use dggal::{DGGAL, DGGRS};
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

pub struct DGGALContext {
    pub _app: Application,
    pub _dggal: DGGAL,
    pub dggrs: DGGRS,
}

fn get_dggal_context() -> DGGALContext {
    let args: Vec<String> = env::args().collect();
    let my_app = Application::new(&args);
    let dggal = DGGAL::new(&my_app);
    let dggrs: DGGRS = DGGRS::new(&dggal, "IVEA3H").expect("Unknown DGGRS");
    DGGALContext {
        _app: my_app,
        _dggal: dggal,
        dggrs,
    }
}

impl DggrsPort for Ivea3hImpl {
    fn zones_from_bbox(&self, depth: u8, densify: bool, bbox: Option<Vec<Vec<f64>>>) -> Zones {
        let ctx = get_dggal_context();
        //let args: Vec<String> = env::args().collect();
        //let my_app = Application::new(&args);
        //let dggal = DGGAL::new(&my_app);
        //let dggrs: DGGRS = DGGRS::new(&dggal, "IVEA3H").expect("Unknown DGGRS");
        let max_depth = ctx.dggrs.getMaxDepth();
        let capped_depth = if depth as i32 > max_depth {
            max_depth
        } else {
            depth as i32
        };

        let zones = if let Some(b) = bbox {
            ctx.dggrs.listZones(capped_depth, &to_geo_extent(Some(b)))
        } else {
            println!("here");
            ctx.dggrs.listZones(
                capped_depth,
                &to_geo_extent(Some(vec![vec![-90.0, -180.0], vec![90.0, 180.0]])),
            )
        };

        ids_to_zones(ctx.dggrs, zones)
    }
    fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Zones {
        let ctx = get_dggal_context();
        let zone = ctx
            .dggrs
            .getZoneFromWGS84Centroid(depth as i32, &to_geo_point(point));
        let zones = vec![zone];
        ids_to_zones(ctx.dggrs, zones)
    }
    fn zones_from_parent(&self, depth: u8, parent_zone_id: String, densify: bool) -> Zones {
        let ctx = get_dggal_context();
        let max_depth = ctx.dggrs.getMaxDepth();

        let capped_depth = if depth as i32 > max_depth {
            max_depth
        } else {
            depth as i32
        };

        let num: u64 = parent_zone_id.parse::<u64>().expect("Invalid u64 string"); // FIX: parent_zone_id needs to be the ZoneID enum not String
        let zones = ctx.dggrs.getSubZones(num, 5);

        ids_to_zones(ctx.dggrs, zones)
    }
    fn zone_from_id(&self, zone_id: String, densify: bool) -> Zones {
        let ctx = get_dggal_context();
        let num: u64 = zone_id.parse::<u64>().expect("Invalid u64 string"); // FIX: parent_zone_id needs to be the ZoneID enum not String
        let zones = vec![num];
        ids_to_zones(ctx.dggrs, zones)
    }
}
