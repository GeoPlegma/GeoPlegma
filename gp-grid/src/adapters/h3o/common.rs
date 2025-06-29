use crate::models::common::{Zone, ZoneID, Zones};
use geo::{Coord, CoordsIter, LineString, Point, Polygon};
use h3o::{Boundary, CellIndex, LatLng, Resolution};

pub fn res(level: u8) -> Resolution {
    Resolution::try_from(level).unwrap_or_else(|_| panic!("Invalid H3 resolution: {}", level))
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

pub fn cells_to_zones(cells: Vec<CellIndex>) -> Zones {
    let zones = cells
        .into_iter()
        .map(|cell| {
            let id = cell.to_string();

            let center = LatLng::from(cell);
            let center_point = latlng_to_point(center); // geo::Point

            let boundary = cell.boundary();
            let region = boundary_to_polygon(&boundary); // geo::Polygon

            let vertex_count = region.exterior().coords_count() as u32;

            let children_opt = match cell.resolution().succ() {
                Some(child_res) => {
                    let children: Vec<String> =
                        cell.children(child_res).map(|c| c.to_string()).collect();
                    Some(children)
                }
                None => {
                    eprintln!("Max resolution reached for cell {}", id);
                    None
                }
            };

            let neighbors_opt = Some(
                cell.grid_disk::<Vec<_>>(1)
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect(),
            );

            Zone {
                id: ZoneID { id },
                region,
                vertex_count,
                center: center_point,
                children: children_opt,
                neighbors: neighbors_opt,
            }
        })
        .collect();

    Zones { zones }
}
