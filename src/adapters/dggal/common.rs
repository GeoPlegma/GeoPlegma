use crate::models::common::{Zone, ZoneID, Zones};
use dggal::{DGGRS, DGGRSZone, GeoExtent, GeoPoint};
use geo::LineString;
use geo::Point;
use geo::Polygon;
use geo::coord;
use std::f64::consts::PI;

pub fn ids_to_zones(dggrs: DGGRS, ids: Vec<DGGRSZone>) -> Zones {
    let zones = ids
        .into_iter()
        .map(|id| {
            let dggal_geo_points: Vec<GeoPoint> = dggrs.getZoneWGS84Vertices(id);
            let region: Polygon<f64> = to_polygon(&dggal_geo_points);

            let center_point = dggrs.getZoneWGS84Centroid(id);
            let center: Point<f64> = to_point(&center_point);

            let count_edges: u32 = dggrs.countZoneEdges(id).try_into().unwrap();

            // TODO: Wrap the children and neighbors into an if statement if requested.
            //let children = dggrs.getSubZones(id, 1);

            let children: Option<Vec<ZoneID>> = Some(
                dggrs
                    .getSubZones(id, 1)
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

            Zone {
                id: ZoneID::IntID(id),
                region,
                vertex_count: count_edges,
                center,
                children, // TODO: we need to make an enum for string and integer based indicies
                neighbors,
            }
        })
        .collect();

    Zones { zones }
}

fn to_point(pt: &GeoPoint) -> Point<f64> {
    Point::new(pt.lon, pt.lat)
}

fn to_polygon(points: &[GeoPoint]) -> Polygon<f64> {
    let mut coords: Vec<_> = points
        .iter()
        .map(|pt| coord! { x: rad_to_deg(pt.lon), y: rad_to_deg(pt.lat) })
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

fn to_string_zone_id(id: DGGRSZone) -> ZoneID {
    ZoneID::StrID(id.to_string())
}

// NOTE: Place this somewhere else
fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

// NOTE: Place this somewhere else
fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / PI
}

pub fn to_geo_point(pt: Point) -> GeoPoint {
    GeoPoint {
        lat: deg_to_rad(pt.y()),
        lon: deg_to_rad(pt.x()),
    }
}

pub fn to_geo_extent(bbox: Option<Vec<Vec<f64>>>) -> GeoExtent {
    match bbox {
        Some(coords) if coords.len() == 2 && coords[0].len() == 2 && coords[1].len() == 2 => {
            let ll = GeoPoint {
                lat: deg_to_rad(coords[0][1]),
                lon: deg_to_rad(coords[0][0]),
            };
            let ur = GeoPoint {
                lat: deg_to_rad(coords[1][1]),
                lon: deg_to_rad(coords[1][0]),
            };
            GeoExtent { ll, ur }
        }
        _ => panic!("Invalid bounding box format"),
    }
}
