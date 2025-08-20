// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::dggal::common::{ids_to_zones, to_geo_extent, to_geo_point};
use crate::error::dggal::DggalError;
use crate::error::port::PortError;
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use dggal::{DGGAL, DGGRS};
use dggal_rust::dggal;
use dggal_rust::ecrt;
use ecrt::Application;
use geo::Point;
use std::env;

// fn get_dggrs(grid_name: &str) -> Result<DGGRS, DggalError> {
//     let args: Vec<String> = env::args().collect();
//     let my_app = Application::new(&args);
//     let dggal = DGGAL::new(&my_app);
//     DGGRS::new(&dggal, grid_name).map_err(|_| DggalError::UnknownGrid {
//         grid_name: grid_name.to_string(),
//     })
// }

pub struct DggalImpl {
    pub grid_name: String,
}

impl DggalImpl {
    pub fn new(grid_name: &str) -> Self {
        Self {
            grid_name: grid_name.to_string(),
        }
    }
}

use crate::adapters::dggal::context::GLOBAL_DGGAL;

fn get_dggrs(grid_name: &str) -> Result<DGGRS, DggalError> {
    let dggal = GLOBAL_DGGAL.lock().map_err(|_| DggalError::LockFailure)?;
    DGGRS::new(&*dggal, grid_name).map_err(|_| DggalError::UnknownGrid {
        grid_name: grid_name.to_string(),
    })
}

impl DggrsPort for DggalImpl {
    fn zones_from_bbox(
        &self,
        depth: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> Result<Zones, PortError> {
        let dggrs = get_dggrs(&self.grid_name)?;

        let max_depth = dggrs.getMaxDepth();
        let capped_depth = if depth as i32 > max_depth {
            max_depth
        } else {
            depth as i32
        };

        let zones = if let Some(b) = bbox {
            dggrs.listZones(capped_depth, &to_geo_extent(Some(b)))
        } else {
            dggrs.listZones(
                capped_depth,
                &to_geo_extent(Some(vec![vec![-90.0, -180.0], vec![90.0, 180.0]])), // FIX: Use the geo Rect struct
            )
        };

        Ok(ids_to_zones(dggrs, zones)?)
    }
    fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Result<Zones, PortError> {
        let dggrs = get_dggrs(&self.grid_name)?;
        let zone = dggrs.getZoneFromWGS84Centroid(depth as i32, &to_geo_point(point));
        let zones = vec![zone];
        Ok(ids_to_zones(dggrs, zones)?)
    }
    fn zones_from_parent(
        &self,
        depth: u8,
        parent_zone_id: String,
        densify: bool,
    ) -> Result<Zones, PortError> {
        let dggrs = get_dggrs(&self.grid_name)?;
        let max_depth = dggrs.getMaxDepth();

        let capped_depth = if depth as i32 > max_depth {
            max_depth
        } else {
            depth as i32
        };

        let num: u64 = parent_zone_id.parse::<u64>().expect("Invalid u64 string"); // FIX: parent_zone_id needs to be the ZoneID enum not String
        let zones = dggrs.getSubZones(num, capped_depth);

        Ok(ids_to_zones(dggrs, zones)?)
    }
    fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<Zones, PortError> {
        let dggrs = get_dggrs(&self.grid_name)?;
        let num: u64 = zone_id.parse::<u64>().expect("Invalid u64 string"); // FIX: parent_zone_id needs to be the ZoneID enum not String
        let zones = vec![num];

        Ok(ids_to_zones(dggrs, zones)?)
    }
}
