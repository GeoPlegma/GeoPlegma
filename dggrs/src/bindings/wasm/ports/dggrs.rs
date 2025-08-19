use std::path::PathBuf;

use serde_wasm_bindgen::from_value;
use wasm_bindgen::prelude::*;

use crate::{
    adapters::dggrid::{
        igeo7::{Igeo7Impl, extract_res_from_cellid, extract_res_from_z3, extract_res_from_z7},
        isea3h::Isea3hImpl,
    },
    bindings::wasm::models::common::JsZones,
    error::port::PortError,
    models::common::Zones,
    ports::dggrs::DggrsPort,
};

// pub enum DggrsPortEnum {
//     Isea3h(Isea3hImpl),
//     Igeo7(Igeo7Impl),
//     // ... future implementors
// }

// impl DggrsPort for DggrsPortEnum {
//     fn zones_from_bbox(
//         &self,
//         depth: u8,
//         densify: bool,
//         bbox: Option<Vec<Vec<f64>>>,
//     ) -> Result<Zones, PortError> {
//         match self {
//             DggrsPortEnum::Isea3h(port) => port.zones_from_bbox(depth, densify, bbox),
//             DggrsPortEnum::Igeo7(port) => port.zones_from_bbox(depth, densify, bbox),
//         }
//     }

//     fn zone_from_point(
//         &self,
//         depth: u8,
//         point: geo::Point,
//         densify: bool,
//     ) -> Result<Zones, PortError> {
//         match self {
//             DggrsPortEnum::Isea3h(port) => port.zone_from_point(depth, point, densify),
//             DggrsPortEnum::Igeo7(port) => port.zone_from_point(depth, point, densify),
//         }
//     }

//     fn zones_from_parent(
//         &self,
//         depth: u8,
//         parent_zone_id: String,
//         densify: bool,
//     ) -> Result<Zones, PortError> {
//         match self {
//             DggrsPortEnum::Isea3h(port) => port.zones_from_parent(depth, parent_zone_id, densify),
//             DggrsPortEnum::Igeo7(port) => port.zones_from_parent(depth, parent_zone_id, densify),
//         }
//     }

//     fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<Zones, PortError> {
//         match self {
//             DggrsPortEnum::Isea3h(port) => port.zone_from_id(zone_id, densify),
//             DggrsPortEnum::Igeo7(port) => port.zone_from_id(zone_id, densify),
//         }
//     }
//     // forward the rest...
// }

// #[wasm_bindgen(js_name = Dggrs)]
// pub struct DggrsPortHandle {
//     inner: DggrsPortEnum,
// }

// #[wasm_bindgen]
// impl DggrsPortHandle {
//     #[wasm_bindgen(constructor)]
//     pub fn new(dggrs: String) -> DggrsPortHandle {
//         DggrsPortHandle {
//             inner: match dggrs.as_str() {
//                 "isea3h" => DggrsPortEnum::Isea3h(Isea3hImpl::new(
//                     PathBuf::from("dggrid"),
//                     PathBuf::from("/dev/shm"),
//                 )),
//                 "igeo7" => DggrsPortEnum::Igeo7(Igeo7Impl::new(
//                     PathBuf::from("dggrid"),
//                     PathBuf::from("/dev/shm"),
//                 )),
//                 _ => panic!("Type a valid DGGRS"),
//             },
//         }
//     }

//     #[wasm_bindgen(js_name = newIsea3h)]
//     pub fn new_isea3h() -> DggrsPortHandle {
//         DggrsPortHandle {
//             inner: DggrsPortEnum::Isea3h(Isea3hImpl::new(
//                 PathBuf::from("dggrid"),
//                 PathBuf::from("/dev/shm"),
//             )),
//         }
//     }

//     #[wasm_bindgen(js_name = newIgeo7)]
//     pub fn new_igeo7() -> DggrsPortHandle {
//         DggrsPortHandle {
//             inner: DggrsPortEnum::Igeo7(Igeo7Impl::new(
//                 PathBuf::from("dggrid"),
//                 PathBuf::from("/dev/shm"),
//             )),
//         }
//     }

//     #[wasm_bindgen(js_name = add)]
//     pub fn add(a: i32, b: i32) -> i32 {
//         a + b
//     }

//     // impl DggrsPort for Isea3hImpl {
//     #[wasm_bindgen(js_name = zonesFromBbox)]
//     pub fn zones_from_bbox(
//         &self,
//         depth: u8,
//         densify: bool,
//         bbox: JsValue,
//     ) -> Result<JsZones, JsValue> {
//         // Convert JsValue -> Option<Vec<Vec<f64>>>
//         let bbox_rust: Option<Vec<Vec<f64>>> = if bbox.is_null() || bbox.is_undefined() {
//             None
//         } else {
//             from_value(bbox).map_err(|e| JsValue::from_str(&format!("bad bbox: {}", e)))?
//         };

//         match self
//             .inner
//             .zones_from_bbox(depth, densify, bbox_rust)
//             .map_err(|e| JsValue::from_str(&e.to_string()))
//         {
//             Ok(z) => {
//                 let zones = z.to_export();
//                 Ok(zones)
//             }
//             Err(err) => Err(err),
//         }
//     }

//     #[wasm_bindgen(js_name = zoneFromPoint)]
//     pub fn zone_from_point(
//         &self,
//         depth: u8,
//         point: JsValue,
//         densify: bool,
//     ) -> Result<JsZones, JsValue> {
//         let sp: Vec<f64> =
//             from_value(point).map_err(|e| JsValue::from_str(&format!("bad point: {}", e)))?;
//         let geo_pt = geo::Point::new(sp[0], sp[1]);

//         match self.inner.zone_from_point(depth, geo_pt, densify) {
//             Ok(z) => {
//                 let zones = z.to_export();
//                 Ok(zones)
//             }
//             Err(err) => Err(JsValue::from_str(&err.to_string())),
//         }
//     }

//     #[wasm_bindgen(js_name = zonesFromParent)]
//     pub fn zones_from_parent(
//         &self,
//         depth: u8,
//         parent_zone_id: String,
//         densify: bool,
//     ) -> Result<JsZones, JsValue> {
//         match self.inner.zones_from_parent(depth, parent_zone_id, densify) {
//             Ok(z) => {
//                 let zones = z.to_export();
//                 Ok(zones)
//             }
//             Err(err) => Err(JsValue::from_str(&err.to_string())),
//         }
//     }

//     #[wasm_bindgen(js_name = zoneFromId)]
//     pub fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<JsZones, JsValue> {
//         match self.inner.zone_from_id(zone_id, densify) {
//             Ok(z) => {
//                 let zones = z.to_export();
//                 Ok(zones)
//             }
//             Err(err) => Err(JsValue::from_str(&err.to_string())),
//         }
//     }
// }
#[wasm_bindgen]
pub fn extract_res_from_cellid_wasm(id: &str, dggs_type: &str) -> Result<u8, String> {
    extract_res_from_cellid(id, dggs_type)
}

#[wasm_bindgen]
/// Extract resolution from ISEA3H ID (Z3)
pub fn extract_res_from_z3_wasm(id: &str) -> Result<u8, String> {
    extract_res_from_z3(id)
}

#[wasm_bindgen]
/// Extract resolution from IGEO7 ID (Z7)
pub fn extract_res_from_z7_wasm(id: &str) -> Result<u8, String> {
    extract_res_from_z7(id)
}
