// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::port::PortError;
use crate::models::common::Zones;
use geo::Point;
// That is the port
pub trait DggrsPort: Send + Sync {
    fn zones_from_bbox(
        &self,
        depth: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> Result<Zones, PortError>;

    fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Result<Zones, PortError>; // NOTE:Consider accepting a vector of Points.
    fn zones_from_parent(
        &self,
        depth: u8,              // FIX: This needs to be relative depth!
        parent_zone_id: String, // FIX: This needs to be ZoneID (so integer or string), see relevant enum.
        densify: bool,
    ) -> Result<Zones, PortError>;
    fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<Zones, PortError>; // NOTE: Consider accepting a vector of ZoneIDs
}
