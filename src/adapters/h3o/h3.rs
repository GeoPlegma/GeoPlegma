use std::str::FromStr;

use crate::adapters::h3o::common::{cells_to_zones, res};
use crate::adapters::h3o::h3o::H3oAdapter;
use crate::models::common::Zones;
use crate::ports::dggrs::DggrsPort;
use geo::{LineString, Point, Polygon};
use h3o::geom::{ContainmentMode, TilerBuilder};
use h3o::{CellIndex, LatLng, Resolution};

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
    fn zones_from_bbox(&self, depth: u8, _densify: bool, bbox: Option<Vec<Vec<f64>>>) -> Zones {
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
                todo!("handle malformed bbox");
            }
        } else {
            // cap res to max 10
            let capped_res = if depth <= 10 { res(depth) } else { res(10) };

            cells = CellIndex::base_cells()
                .flat_map(|base| base.children(capped_res))
                .collect::<Vec<_>>();
        }
        println!("{}", cells.len());
        let zones = cells_to_zones(cells);
        zones
    }
    fn zone_from_point(&self, depth: u8, point: Point, _densify: bool) -> Zones {
        let coord = LatLng::new(point.x(), point.y()).expect("valid coord");

        let cell = coord.to_cell(res(depth)); // ToDo: we should support multiple points at once.

        let zones = cells_to_zones(vec![cell]);
        zones
    }
    fn zones_from_parent(
        &self,
        depth: u8,
        zone_id: String, // ToDo: needs validation function
        _densify: bool,
    ) -> Zones {
        let parent = match CellIndex::from_str(&zone_id) {
            Ok(cell) => cell,
            Err(_) => {
                eprintln!("Invalid zone_id: {}", zone_id);
                return Zones { zones: vec![] }; // or handle error differently
            }
        };

        let base_res = parent.resolution();

        let target_res = match Resolution::try_from(u8::from(base_res) + depth) {
            Ok(res) => res,
            Err(_) => {
                eprintln!("Resolution exceeds max allowed (15)");
                return Zones { zones: vec![] };
            }
        };

        let children: Vec<CellIndex> = parent.children(target_res).collect();

        cells_to_zones(children)
    }
    fn zone_from_id(
        &self,
        zone_id: String, // ToDo: needs validation function
        _densify: bool,
    ) -> Zones {
        let zone = match CellIndex::from_str(&zone_id) {
            Ok(cell) => cell,
            Err(_) => {
                eprintln!("Invalid zone_id: {}", zone_id);
                return Zones { zones: vec![] }; // or handle error differently
            }
        };

        let zone_res = zone.resolution();

        cells_to_zones(vec![zone])
    }
}
