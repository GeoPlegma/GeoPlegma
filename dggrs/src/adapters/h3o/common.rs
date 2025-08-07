// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    error::{h3o::H3oError, port::GeoPlegmaError},
    models::common::{Zone, ZoneId, Zones},
};
use geo::{Coord, CoordsIter, LineString, Point, Polygon};
use h3o::{Boundary, CellIndex, LatLng, Resolution};

/// Translates integer resolution to H3 string resolution
pub fn res(depth: u8) -> Resolution {
    Resolution::try_from(depth).unwrap_or_else(|_| panic!("Invalid H3 depth: {}", depth))
}

pub fn boundary_to_polygon(boundary: &Boundary) -> Polygon<f64> {
    let mut coords: Vec<Coord<f64>> = boundary
        .iter()
        .map(|latlng| Coord {
            x: latlng.lng(),
            y: latlng.lat(),
        })
        .collect();

    // Ensure the ring is closed
    if coords.first() != coords.last() {
        if let Some(first) = coords.first().copied() {
            coords.push(first);
        }
    }

    Polygon::new(LineString::from(coords), vec![])
}

pub fn children_to_strings(iter: impl Iterator<Item = CellIndex>) -> Vec<String> {
    iter.map(|cell| cell.to_string()).collect()
}

pub fn ring_to_strings(iter: impl Iterator<Item = Option<CellIndex>>) -> Vec<String> {
    iter.filter_map(|opt| opt.map(|cell| cell.to_string()))
        .collect()
}

pub fn latlng_to_point(latlng: LatLng) -> Point {
    Point::new(latlng.lng(), latlng.lat())
}

pub fn cells_to_zones(cells: Vec<CellIndex>) -> Result<Zones, GeoPlegmaError> {
    let zones: Vec<Zone> = cells
        .into_iter()
        .map(|cell| {
            let id = ZoneId::new_hex(&cell.to_string())?;

            let center = LatLng::from(cell);
            let center_point = latlng_to_point(center); // geo::Point

            let boundary = cell.boundary();
            let region = boundary_to_polygon(&boundary); // geo::Polygon

            let vertex_count = region.exterior().coords_count() as u32;

            let child_res = cell
                .resolution()
                .succ() // succ() returns an Option, therefore we can use ok_or_else in the next line and not map_err
                .ok_or_else(|| H3oError::ResolutionLimitReached {
                    zone_id: cell.to_string(),
                })?;

            let children_opt: Vec<ZoneId> = cell
                .children(child_res)
                .map(|c| ZoneId::new_hex(&c.to_string()))
                .collect::<Result<_, _>>()?; // NOTE: In Result<_ , _>> the _ means that the T and E are inferred. 

            let neighbors_opt: Vec<ZoneId> = cell
                .grid_disk::<Vec<CellIndex>>(1)
                .into_iter()
                .map(|c| ZoneId::new_hex(&c.to_string()))
                .collect::<Result<_, _>>()?; // NOTE: In Result<_ , _>> the _ means that the T and E are inferred. 

            Ok(Zone {
                id,
                region,
                vertex_count,
                center: center_point,
                children: Some(children_opt),
                neighbors: Some(neighbors_opt),
            })
        })
        .collect::<Result<Vec<Zone>, GeoPlegmaError>>()?;

    Ok(Zones { zones })
}
