// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::dggal::common::ids_to_zones;
use crate::adapters::dggal::dggal::DggalAdapter;
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use geo::{LineString, Point, Polygon};
extern crate ecrt;
use ecrt::{Application, tokenizeWith};
extern crate dggal;
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

        let zones = if let Some(b) = bbox {
            todo!("there is an issue with bounding boxes in dggal");
        } else {
            if depth as i32 > max_depth {
                d.listZones(max_depth, &wholeWorld)
            } else {
                d.listZones(depth as i32, &wholeWorld)
            }
        };

        println!(
            "The length of the array of zone IDs for the whole world: \n{:?}\n\n",
            zones.len()
        );

        println!("test {:?}", max_depth);

        ids_to_zones(d, zones)
    }
    fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Zones {
        todo!("Not done");
    }
    fn zones_from_parent(
        &self,
        depth: u8,
        parent_zone_id: String,
        // clip_cell_res: u8,
        densify: bool,
    ) -> Zones {
        todo!("Not done");
    }
    fn zone_from_id(&self, zone_id: String, densify: bool) -> Zones {
        todo!("Not done");
    }
}
