// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::dggal::DggalError;
use crate::models::common::{Zone, ZoneId, Zones};
use dggal_rust::dggal::{DGGRS, DGGRSZone, GeoExtent, GeoPoint};
use geo::{LineString, Point, Polygon, Rect, coord};

pub fn ids_to_zones(dggrs: DGGRS, ids: Vec<DGGRSZone>) -> Result<Zones, DggalError> {
    let zones: Vec<Zone> = ids
        .into_iter()
        .map(|id| {
            //let dggal_geo_points: Vec<GeoPoint> = dggrs.getZoneWGS84Vertices(id);
            let dggal_geo_points: Vec<GeoPoint> = dggrs.getZoneRefinedWGS84Vertices(id, 0);
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

            let children: Option<Vec<ZoneId>> = Some(
                dggrs
                    .getZoneChildren(id)
                    .into_iter()
                    .map(to_u64_zone_id)
                    .collect(),
            );

            let mut nb_types: [i32; 6] = [0; 6];
            //let neighbors = dggrs.getZoneNeighbors(id, &mut nb_types);

            let neighbors: Option<Vec<ZoneId>> = Some(
                dggrs
                    .getZoneNeighbors(id, &mut nb_types)
                    .into_iter()
                    .map(to_u64_zone_id)
                    .collect(),
            );

            Ok(Zone {
                id: ZoneId::IntId(id),
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

fn to_u64_zone_id(id: DGGRSZone) -> ZoneId {
    // NOTE: Expand this to do the conversion automaticallt
    ZoneId::IntId(id)
}

pub fn to_geo_point(pt: Point) -> GeoPoint {
    GeoPoint {
        lat: pt.y().to_radians(),
        lon: pt.x().to_radians(),
    }
}

/// Convert geo::Rect BBox to DGGAL::GeoExtent
pub fn bbox_to_geoextent(bbox: &Rect<f64>) -> GeoExtent {
    let min = bbox.min(); // lower-left in degrees
    let max = bbox.max(); // upper-right in degrees

    GeoExtent {
        ll: GeoPoint {
            lat: min.y.to_radians(),
            lon: min.x.to_radians(),
        },
        ur: GeoPoint {
            lat: max.y.to_radians(),
            lon: max.x.to_radians(),
        },
    }
}
