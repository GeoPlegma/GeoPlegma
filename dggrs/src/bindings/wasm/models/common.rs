use geo::{LineString, Point, Polygon};
use std::{collections::HashMap, fmt};
use wasm_bindgen::prelude::*;

use crate::{
    models::common::{Zone, ZoneID, Zones},
    wasm_fields_clone,
};

/// The Zone struct has nested heap allocations (String, Vec<(f64,f64)>, Vec<String>), which means:
/// Each String is 24 bytes (ptr, len, capacity) + heap data.
/// Each (f64, f64) is fine in Rust, but Vec<(f64,f64)> is not a flat Vec<f64> in wasm.
/// wasm-bindgen will have to walk and serialize everything, which is slow for thousands of zones.

/// No wasm_bindgen overhead per zone — you pass one pointer + length per field instead of millions of small objects.
/// Zero-copy — JS reads directly from WebAssembly memory.
/// Keeps geometry-heavy Zone struct in Rust for efficient calculations.
/// Scales to millions of zones without crashing the browser or blowing up memory usage.
#[wasm_bindgen]
pub struct JsZones {
    // zone ids flattened
    id_offsets: Vec<u32>, // len = num_zones (start index of each id in utf8_ids)
    utf8_ids: Vec<u8>,

    // centers
    center_x: Vec<f64>,
    center_y: Vec<f64>,

    // vertex counts
    vertex_count: Vec<u32>,

    // regions (flattened coordinates)
    region_offsets: Vec<u32>, // len = num_zones (start index of each zone's coords in region_coords)
    region_coords: Vec<f64>,  // flattened x,y,x,y,...

    // children as indices into `zones` vector
    children_offsets: Vec<u32>, // len = num_zones (start index into children_index)
    children_index: Vec<u32>,   // flattened child indices

    // neighbors as indices into `zones` vector
    neighbors_offsets: Vec<u32>, // len = num_zones (start index into neighbors_index)
    neighbors_index: Vec<u32>,   // flattened neighbor indices
}

wasm_fields_clone!(
    JsZones,
    (get_id_offsets, set_id_offsets, id_offsets, "id_offsets", Vec<u32>),
    (get_utf8_ids, set_utf8_ids, utf8_ids, "utf8_ids", Vec<u8>),
    (get_center_x, set_center_x, center_x, "center_x", Vec<f64>),
    (get_center_y, set_center_y, center_y, "center_y", Vec<f64>),
    (get_vertex_count, set_vertex_count, vertex_count, "vertex_count", Vec<u32>),
    (get_region_offset, set_region_offset, region_offsets, "region_offset", Vec<u32>),
    (get_region_coords, set_region_coords, region_coords, "region_coords", Vec<f64>),
    (get_children_offsets, set_children_offsets, children_offsets, "children_offsets",Vec<u32>),
    (get_children_index, set_children_index, children_index, "children_index",Vec<u32>),
    (get_neighbors_offsets, set_neighbors_offsets, neighbors_offsets, "neighbors_offsets",Vec<u32>),
    (get_neighbors_index, set_neighbors_index, neighbors_index, "neighbors_index",Vec<u32>));

impl JsZones {
    /// Rebuild a `Zones` struct from a flattened `ZonesExport`
    pub fn to_import(&self) -> Zones {
        let zone_count = self.id_offsets.len();
        let mut zones = Vec::with_capacity(zone_count);

        // 1) reconstruct id strings
        let mut ids: Vec<String> = Vec::with_capacity(zone_count);
        for i in 0..zone_count {
            let start = self.id_offsets[i] as usize;
            let end = if i + 1 < zone_count {
                self.id_offsets[i + 1] as usize
            } else {
                self.utf8_ids.len()
            };
            let s =
                str::from_utf8(&self.utf8_ids[start..end]).expect("invalid utf8 in id buffer");
            ids.push(s.to_string());
        }
        // 2) build zones
        for i in 0..zone_count {
            // region
            let region_start = self.region_offsets[i] as usize;
            let region_end = if i + 1 < zone_count {
                self.region_offsets[i + 1] as usize
            } else {
                self.region_coords.len()
            };
            let mut coords = Vec::new();
            let mut j = region_start;
            while j + 1 < region_end {
                coords.push((self.region_coords[j], self.region_coords[j + 1]));
                j += 2;
            }
            let line_string: LineString = coords.into();
            let region: Polygon = Polygon::new(line_string, vec![]);

            // children
            let children_start = self.children_offsets[i] as usize;
            let children_end = if i + 1 < zone_count {
                self.children_offsets[i + 1] as usize
            } else {
                self.children_index.len()
            };
            let children: Option<Vec<ZoneID>> = if children_end > children_start {
                Some(
                    self.children_index[children_start..children_end]
                        .iter()
                        .map(|&idx| ZoneID::StrID(ids[idx as usize].clone()))
                        .collect(),
                )
            } else {
                None
            };

            // neighbors
            let neighbors_start = self.neighbors_offsets[i] as usize;
            let neighbors_end = if i + 1 < zone_count {
                self.neighbors_offsets[i + 1] as usize
            } else {
                self.neighbors_index.len()
            };
            let neighbors: Option<Vec<ZoneID>> = if neighbors_end > neighbors_start {
                Some(
                    self.neighbors_index[neighbors_start..neighbors_end]
                        .iter()
                        .map(|&idx| ZoneID::StrID(ids[idx as usize].clone()))
                        .collect(),
                )
            } else {
                None
            };

            zones.push(Zone {
                id: ZoneID::StrID(ids[i].clone()),
                region: region,
                center: Point::new(self.center_x[i], self.center_y[i]),
                vertex_count: self.vertex_count[i],
                children,
                neighbors,
            })
        }

        Zones { zones: zones }
    }
}

// @TODO needs to be reviewed
impl Zones {
    /// Flatten `Zones` into `ZonesExport`:
    /// - Ids are concatenated into utf8_ids w/ id_offsets
    /// - Centers, vertex_count repeated per zone
    /// - Regions flattened with region_offsets
    /// - children/neighbors represented as indices into the zone list
    pub fn to_export(&self) -> JsZones {
        let n = self.zones.len();

        let mut id_offsets = Vec::with_capacity(n);
        let mut utf8_ids = Vec::new();

        let mut center_x = Vec::with_capacity(n);
        let mut center_y = Vec::with_capacity(n);
        let mut vertex_count = Vec::with_capacity(n);

        let mut region_offsets = Vec::with_capacity(n);
        let mut region_coords = Vec::new();

        // Pre-pass: collect id strings and build id -> index map
        for (i, zone) in self.zones.iter().enumerate() {
            // size of ids
            id_offsets.push(utf8_ids.len() as u32);
            // ids array
            let id_str = zone.id.to_string(); // ZoneID implements Display
            utf8_ids.extend_from_slice(id_str.as_bytes());
            // optionally add a separator if you need readable boundaries, but offsets suffice
            // no separator to save space
        }
        // Build mapping from string id -> index
        let mut id_to_index: HashMap<String, u32> = HashMap::with_capacity(n);
        for (i, zone) in self.zones.iter().enumerate() {
            id_to_index.insert(zone.id.to_string(), i as u32);
        }

        // Second pass for centers, vertex counts, regions, children/neighbors
        let mut children_offsets = Vec::with_capacity(n);
        let mut children_index = Vec::new();

        let mut neighbors_offsets = Vec::with_capacity(n);
        let mut neighbors_index = Vec::new();

        for zone in &self.zones {
            // centers & vertex_count
            center_x.push(zone.center.x());
            center_y.push(zone.center.y());
            vertex_count.push(zone.vertex_count);

            // region exterior ring flattened (x,y)
            region_offsets.push(region_coords.len() as u32);
            // Use exterior ring points (you may want interior rings too depending on your data)
            for coord in zone.region.exterior().points() {
                region_coords.push(coord.x());
                region_coords.push(coord.y());
            }

            // children -> indices
            children_offsets.push(children_index.len() as u32);
            if let Some(children) = &zone.children {
                for child_id in children {
                    let idx_opt = id_to_index.get(&child_id.to_string());
                    if let Some(idx) = idx_opt {
                        children_index.push(*idx);
                    } else {
                        // choose desired behavior if child id not found: skip or push u32::MAX
                        // here we skip missing children
                    }
                }
            }

            // neighbors -> indices
            neighbors_offsets.push(neighbors_index.len() as u32);
            if let Some(neighbors) = &zone.neighbors {
                for neigh_id in neighbors {
                    let idx_opt = id_to_index.get(&neigh_id.to_string());
                    if let Some(idx) = idx_opt {
                        neighbors_index.push(*idx);
                    } else {
                        // skip missing neighbor
                    }
                }
            }
        }

        JsZones {
            id_offsets,
            utf8_ids,
            center_x,
            center_y,
            vertex_count,
            region_offsets,
            region_coords,
            children_offsets,
            children_index,
            neighbors_offsets,
            neighbors_index,
        }
    }
}
