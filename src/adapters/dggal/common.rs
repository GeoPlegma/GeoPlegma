// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::dggal::DggalError;
use crate::models::common::{Zone, ZoneID, Zones};
use dggal_rust::dggal::{DGGRS, DGGRSZone, GeoExtent, GeoPoint};
use geo::LineString;
use geo::Point;
use geo::Polygon;
use geo::coord;

pub fn ids_to_zones(dggrs: DGGRS, ids: Vec<DGGRSZone>) -> Result<Zones, DggalError> {
    let zones: Vec<Zone> = ids
        .into_iter()
        .map(|id| {
            let dggal_geo_points: Vec<GeoPoint> = dggrs.getZoneWGS84Vertices(id);
            let region: Polygon<f64> = to_polygon(&dggal_geo_points);

            let center_point = dggrs.getZoneWGS84Centroid(id);
            let center: Point<f64> = to_point(&center_point);

            let count_edges: u32 = dggrs.countZoneEdges(id).try_into().unwrap(); // NOTE: Implement proper error ahndling in error.rs (to be created) and do pattern matching here. 
            //
            //
            let count_edges: u32 = dggrs.countZoneEdges(id).try_into().map_err(|e| {
                DggalError::EdgeCountConversion {
                    zone_id: id.to_string(),
                    source: e,
                }
            })?;

            // TODO: Wrap the children and neighbors into an if statement if requested.
            //let children = dggrs.getSubZones(id, 1);

            let children: Option<Vec<ZoneID>> = Some(
                dggrs
                    .getZoneChildren(id)
                    .into_iter()
                    .map(to_u64_zone_id)
                    .collect(),
            );

            let mut nb_types: [i32; 6] = [0; 6];
            //let neighbors = dggrs.getZoneNeighbors(id, &mut nb_types);

            let neighbors: Option<Vec<ZoneID>> = Some(
                dggrs
                    .getZoneNeighbors(id, &mut nb_types)
                    .into_iter()
                    .map(to_u64_zone_id)
                    .collect(),
            );

            Ok(Zone {
                id: ZoneID::IntID(id),
                region,
                vertex_count: count_edges,
                center,
                children, // TODO: we need to make an enum for string and integer based indicies
                neighbors,
            })
        })
        .collect::<Result<Vec<Zone>, DggalError>>()?;

    Ok(Zones { zones })
}

fn to_point(pt: &GeoPoint) -> Point<f64> {
    Point::new(pt.lon, pt.lat)
}

fn to_polygon(points: &[GeoPoint]) -> Polygon<f64> {
    let mut coords: Vec<_> = points
        .iter()
        .map(|pt| coord! { x: pt.lon.to_degrees(), y: pt.lat.to_degrees() })
        .collect();

    if coords.first() != coords.last() {
        coords.push(coords[0]);
    }

    Polygon::new(LineString::from(coords), vec![])
}

fn to_u64_zone_id(id: DGGRSZone) -> ZoneID {
    // NOTE: Expand this to do the conversion automaticallt
    ZoneID::IntID(id)
}

pub fn to_geo_point(pt: Point) -> GeoPoint {
    GeoPoint {
        lat: pt.y().to_radians(),
        lon: pt.x().to_radians(),
    }
}

pub fn to_geo_extent(bbox: Option<Vec<Vec<f64>>>) -> GeoExtent {
    match bbox {
        Some(coords) if coords.len() == 2 && coords[0].len() == 2 && coords[1].len() == 2 => {
            let ll = GeoPoint {
                lat: coords[0][1].to_radians(),
                lon: coords[0][0].to_radians(),
            };
            let ur = GeoPoint {
                lat: coords[1][1].to_radians(),
                lon: coords[1][0].to_radians(),
            };
            GeoExtent { ll, ur }
        }
        _ => panic!("Invalid bounding box format"), // FIX: remove panic. bbox: Option<Vec<Vec<f64>>> has to be replaced with geo::geometry::Rect here https://docs.rs/geo/latest/geo/geometry/struct.Rect.html
    }
}
