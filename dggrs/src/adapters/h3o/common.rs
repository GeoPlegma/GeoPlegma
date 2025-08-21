// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    error::{h3o::H3oError, port::GeoPlegmaError},
    models::common::{RefinementLevel, Zone, ZoneId, Zones},
    ports::dggrs::DggrsPortConfig,
};
use geo::{Coord, CoordsIter, GeodesicArea, LineString, Point, Polygon};
use h3o::{Boundary, CellIndex, LatLng, Resolution};

/// Translates integer resolution to H3 string resolution
pub fn refinement_level_to_h3_resolution(
    refinement_level: RefinementLevel,
) -> Result<Resolution, GeoPlegmaError> {
    Resolution::try_from(refinement_level.get()).map_err(|e| {
        GeoPlegmaError::H3o(H3oError::CannotTranslateToH3Resolution {
            rf: refinement_level.to_string(),
            source: e,
        })
    })
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

pub fn to_zones(h3o_zones: Vec<CellIndex>, conf: DggrsPortConfig) -> Result<Zones, GeoPlegmaError> {
    let zones: Vec<Zone> = h3o_zones
        .into_iter()
        .map(|h3o_zone| {
            let id = ZoneId::new_hex(&h3o_zone.to_string())?;

            let center = if conf.center {
                let ll = LatLng::from(h3o_zone);
                Some(latlng_to_point(ll)) // geo::Point
            } else {
                None
            };

            let region = if conf.region || conf.area_sqm || conf.vertex_count {
                let boundary = h3o_zone.boundary();
                Some(boundary_to_polygon(&boundary)) // geo::Polygon
            } else {
                None
            };

            let area_sqm = if conf.area_sqm {
                region.as_ref().map(|r| r.geodesic_area_unsigned()) // NOTE: It is also an option to use the build in area function of H3o
            } else {
                None
            };

            let vertex_count = if conf.vertex_count {
                region.as_ref().map(|r| r.exterior().coords_count() as u32) // NOTE: It is also an option to use the build-in vertex function of H3o
            } else {
                None
            };

            let children = if conf.children {
                let chr_res = h3o_zone
                    .resolution()
                    .succ() // NOTE: succ() returns an Option, therefore we can use ok_or_else in the next line and not map_err
                    .ok_or_else(|| H3oError::ResolutionLimitReached {
                        zone_id: h3o_zone.to_string(),
                    })?;

                let chr_vec: Vec<ZoneId> = h3o_zone
                    .children(chr_res)
                    .map(|c| ZoneId::new_hex(&c.to_string()))
                    .collect::<Result<_, _>>()?; // NOTE: In Result<_ , _>> the _ means that the T and E are inferred. 
                Some(chr_vec)
            } else {
                None
            };

            let neighbors = if conf.neighbors {
                let nbr: Vec<ZoneId> = h3o_zone
                    .grid_disk::<Vec<CellIndex>>(1)
                    .into_iter()
                    .map(|c| ZoneId::new_hex(&c.to_string()))
                    .collect::<Result<_, _>>()?; // NOTE: In Result<_ , _>> the _ means that the T and E are inferred. 
                Some(nbr)
            } else {
                None
            };

            Ok(Zone {
                id,
                region,
                center,
                vertex_count,
                children,
                neighbors,
                area_sqm,
            })
        })
        .collect::<Result<Vec<Zone>, GeoPlegmaError>>()?;

    Ok(Zones { zones })
}
