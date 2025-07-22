// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::h3o::common::{cells_to_zones, res};
use crate::adapters::h3o::h3o::H3oAdapter;
use crate::error::h3o::H3oError;
use crate::error::port::PortError;
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use geo::{LineString, Point, Polygon};
use h3o::geom::{ContainmentMode, TilerBuilder};
use h3o::{CellIndex, LatLng, Resolution};
use std::str::FromStr;

pub const MAX_DEPTH: u8 = 10;

pub struct H3Impl {
    pub adapter: H3oAdapter,
}

impl H3Impl {
    pub fn new() -> Self {
        Self {
            adapter: H3oAdapter::new(),
        }
    }
}

impl Default for H3Impl {
    fn default() -> Self {
        Self {
            adapter: H3oAdapter::default(),
        }
    }
}

impl DggrsPort for H3Impl {
    fn zones_from_bbox(
        &self,
        depth: u8,
        _densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> Result<Zones, PortError> {
        let cells: Vec<CellIndex>;

        let mut tiler = TilerBuilder::new(res(2))
            .containment_mode(ContainmentMode::Covers)
            .build();

        if let Some(b) = bbox {
            // Validate bbox format: [[minX, minY], [maxX, maxY]]
            if b.len() == 2 && b[0].len() == 2 && b[1].len() == 2 {
                let (minx, miny) = (b[0][0], b[0][1]);
                let (maxx, maxy) = (b[1][0], b[1][1]);

                // Create a counter-clockwise ring (geo expects CCW)
                let ring = LineString::from(vec![
                    (minx, miny),
                    (maxx, miny),
                    (maxx, maxy),
                    (minx, maxy),
                    (minx, miny),
                ]);

                let polygon = Polygon::new(ring, vec![]);
                let _ = tiler.add(polygon);
                cells = tiler.into_coverage().collect::<Vec<_>>();
            } else {
                todo!("handle malformed bbox"); // TODO: Should be fixed with proper bbox
            }
        } else {
            // cap res to max 10
            let capped_res = if depth <= 10 { res(depth) } else { res(10) };

            cells = CellIndex::base_cells()
                .flat_map(|base| base.children(capped_res))
                .collect::<Vec<_>>();
        }
        Ok(cells_to_zones(cells)?)
    }
    fn zone_from_point(&self, depth: u8, point: Point, _densify: bool) -> Result<Zones, PortError> {
        let coord = LatLng::new(point.x(), point.y()).expect("valid coord");

        let cell = coord.to_cell(res(depth)); // TODO: we should support multiple points at once.

        //let zones = cells_to_zones(vec![cell]);
        Ok(cells_to_zones(vec![cell])?)
    }
    fn zones_from_parent(
        &self,
        depth: u8,
        zone_id: String, // ToDo: needs validation function
        _densify: bool,
    ) -> Result<Zones, PortError> {
        let parent = CellIndex::from_str(&zone_id).map_err(|e| {
            PortError::H3o(H3oError::InvalidZoneID {
                zone_id: zone_id.clone(),
                source: e,
            })
        })?;

        let base_res = parent.resolution();
        let next = u8::from(base_res) + depth;

        let target_res = Resolution::try_from(next).map_err(|e| {
            PortError::H3o(H3oError::InvalidResolution {
                zone_id: zone_id.clone(),
                depth,
                source: e,
            })
        })?;

        let children: Vec<CellIndex> = parent.children(target_res).collect();

        Ok(cells_to_zones(children)?)
    }
    fn zone_from_id(
        &self,
        zone_id: String, // ToDo: needs validation function
        _densify: bool,
    ) -> Result<Zones, PortError> {
        let cell = CellIndex::from_str(&zone_id).map_err(|e| {
            PortError::H3o(H3oError::InvalidZoneID {
                zone_id: zone_id.clone(),
                source: e,
            })
        })?;

        Ok(cells_to_zones(vec![cell])?)
    }
}
