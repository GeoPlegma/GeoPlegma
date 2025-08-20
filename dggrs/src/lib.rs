// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc = include_str!("../../README.md")]
pub mod adapters;
pub mod bindings;
pub mod dggrs;
pub mod error;
pub mod factory;
pub mod models;
pub mod ports;

use std::{env, path::PathBuf};

/// This is the only re-export that is needed.
pub use factory::dggrs_factory::get;

// mod macros;

// use napi::{bindgen_prelude::{Array, Float32Array}, Error};
// use napi_derive::napi;
// use serde_wasm_bindgen::{from_value, to_value};
// use wasm_bindgen::prelude::*;

// use crate::{
//     adapters::dggrid::{igeo7::Igeo7Impl, isea3h::Isea3hImpl},
//     bindings::wasm::models::common::JsZones,
//     error::port::PortError,
//     models::common::Zones,
//     ports::dggrs::DggrsPort,
// };

// // Import functions from a JS module that will live next to the generated glue (pkg/).
// // We use a relative specifier that matches where wasm-pack puts dggrs.js.
// #[wasm_bindgen(module = "/node_fs.js")]
// extern "C" {
//     // Use `catch` so JS exceptions become `Result<_, JsValue>` in Rust.
//     #[wasm_bindgen(catch)]
//     fn append_to_file(path: &str, contents: &str) -> Result<(), JsValue>;

//     #[wasm_bindgen(catch)]
//     fn read_file(path: &str) -> Result<String, JsValue>;
// }
// // #[wasm_bindgen]
// // #[napi]
// // pub fn add(a: i32, b: i32) -> i32 {
// //     a + b
// // }
// pub fn append_to_file_wasm(path: &str, contents: &str) -> Result<(), JsValue> {
//     append_to_file(path, contents)
// }
// pub fn read_file_wasm(path: &str) -> Result<String, JsValue> {
//     read_file(path)
// }
// // Export a struct with methods
// #[wasm_bindgen]
// pub struct Greeter {
//     name: String,
// }

// #[wasm_bindgen]
// impl Greeter {
//     #[wasm_bindgen(constructor)]
//     pub fn new(name: String) -> Greeter {
//         Greeter { name }
//     }

//     pub fn greet(&self) -> String {
//         format!("Hello, {}!", self.name)
//     }
// }

// use geo::Point;
// // That is the port
// // pub trait DggrsPort: Send + Sync {
// //     fn zones_from_bbox(
// //         &self,
// //         depth: u8,
// //         densify: bool,
// //         bbox: Option<Vec<Vec<f64>>>,
// //     ) -> Result<Zones, PortError>;

// //     fn zone_from_point(&self, depth: u8, point: Point, densify: bool) -> Result<Zones, PortError>; // NOTE:Consider accepting a vector of Points.
// //     fn zones_from_parent(
// //         &self,
// //         depth: u8,              // FIX: This needs to be relative depth!
// //         parent_zone_id: String, // FIX: This needs to be ZoneID (so integer or string), see relevant enum.
// //         densify: bool,
// //     ) -> Result<Zones, PortError>;
// //     fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<Zones, PortError>; // NOTE: Consider accepting a vector of ZoneIDs
// // }
// // #[wasm_bindgen(js_name = Dggrs)]
// #[napi]
// pub struct Dggrs {
//     inner: DggrsPortEnum,
// }

// pub enum DggrsPortEnum {
//     Isea3h(Isea3hImpl),
//     Igeo7(Igeo7Impl),
//     // ... future implementors
// }

// impl DggrsPort for DggrsPortEnum {
//     fn zones_from_bbox1(
//         &self,
//         depth: u8,
//         densify: bool,
//         bbox: Option<Vec<Vec<f64>>>,
//     ) -> Result<Zones, PortError> {
//         match self {
//             DggrsPortEnum::Isea3h(port) => port.zones_from_bbox1(depth, densify, bbox),
//             DggrsPortEnum::Igeo7(port) => port.zones_from_bbox1(depth, densify, bbox),
//         }
//     }

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

// // #[wasm_bindgen]
// #[napi]
// impl Dggrs {
//     // #[wasm_bindgen(constructor)]
//     #[napi(constructor)]
//     pub fn new(dggrs: String) -> Dggrs {
//         Dggrs {
//             inner: match dggrs.as_str() {
//                 "isea3h" => DggrsPortEnum::Isea3h(Isea3hImpl::new(
//                     PathBuf::from("dggrid"),
//                     PathBuf::from(""),
//                 )),
//                 "igeo7" => {
//                     DggrsPortEnum::Igeo7(Igeo7Impl::new(PathBuf::from("dggrid"), PathBuf::from("")))
//                 }
//                 _ => panic!("Type a valid DGGRS"),
//             },
//         }
//     }

//     // #[wasm_bindgen(js_name = add)]
//     #[napi(js_name = add)]
//     pub fn add(&self, a: i32, b: i32) -> i32 {
//         a + b
//     }

//     // ========================
//     // #[wasm_bindgen(js_name = zonesFromBbox1)]
//     #[napi(js_name = zonesFromBbox1)]
//     pub fn zones_from_bbox(
//         &self,
//         depth: u8,
//         densify: bool,
//         bbox: Option<Vec<Vec<f64>>>,
//     ) -> napi::Result<JsZones> {
//         // Convert JsValue -> Option<Vec<Vec<f64>>>
//         // let bbox_rust: Option<Vec<Vec<f64>>> = if bbox.is_null() || bbox.is_undefined() {
//         //     None
//         // } else {
//         //     from_value(bbox).map_err(|e| JsValue::from_str(&format!("bad bbox: {}", e)))?
//         // };
//     //       let bbox_rust: Option<Vec<Vec<f64>>> = match bbox {
//     //     Some(arr) => {
//     //         let mut outer: Vec<Vec<f64>> = Vec::new();
//     //         let len = arr.len() as u32;

//     //         for i in 0..len {
//     //             let inner_val: JsUnknown = arr.get(i)?;
//     //             let inner_arr: Array = inner_val.coerce_to_object()?.coerce_to_array()?;
//     //             let inner_len = inner_arr.len()? as u32;

//     //             let mut inner_vec: Vec<f64> = Vec::new();
//     //             for j in 0..inner_len {
//     //                 let num_val: JsNumber = inner_arr.get(j)?;
//     //                 inner_vec.push(num_val.get_double()?);
//     //             }
//     //             outer.push(inner_vec);
//     //         }

//     //         Some(outer)
//     //     }
//     //     None => None,
//     // };
//     let zones = self
//         .inner
//         .zones_from_bbox(depth, densify, bbox)
//         .map_err(|e| Error::from_reason(e.to_string()))?;

//     // Ok( JsZones::from(&zones))
//     Ok(zones.to_export())
//         // match self
//         //     .inner
//         //     .zones_from_bbox1(depth, densify, bbox)
//         //     .map_err(|e| JsValue::from_str(&e.to_string()))
//         // {
//         //     Ok(z) => {
//         //         let zones = z.to_export();
//         //         Ok(zones)
//         //     }
//         //     Err(err) => Err(err),
//         // }
//     }
//     // ========================
//     // impl DggrsPort for Isea3hImpl {
//     // #[wasm_bindgen(js_name = zonesFromBbox)]
//     // pub fn zones_from_bbox(
//     //     &self,
//     //     depth: u8,
//     //     densify: bool,
//     //     bbox: JsValue,
//     // ) -> Result<JsZones, JsValue> {
//     //     // Convert JsValue -> Option<Vec<Vec<f64>>>
//     //     let bbox_rust: Option<Vec<Vec<f64>>> = if bbox.is_null() || bbox.is_undefined() {
//     //         None
//     //     } else {
//     //         from_value(bbox).map_err(|e| JsValue::from_str(&format!("bad bbox: {}", e)))?
//     //     };

//     //     match self
//     //         .inner
//     //         .zones_from_bbox(depth, densify, bbox_rust)
//     //         .map_err(|e| JsValue::from_str(&e.to_string()))
//     //     {
//     //         Ok(z) => {
//     //             let zones = z.to_export();
//     //             Ok(zones)
//     //         }
//     //         Err(err) => Err(err),
//     //     }
//     // }

//     // #[wasm_bindgen(js_name = zoneFromPoint)]
//     // pub fn zone_from_point(
//     //     &self,
//     //     depth: u8,
//     //     point: JsValue,
//     //     densify: bool,
//     // ) -> Result<JsZones, JsValue> {
//     //     let sp: Vec<f64> =
//     //         from_value(point).map_err(|e| JsValue::from_str(&format!("bad point: {}", e)))?;
//     //     let geo_pt = geo::Point::new(sp[0], sp[1]);

//     //     match self.inner.zone_from_point(depth, geo_pt, densify) {
//     //         Ok(z) => {
//     //             let zones = z.to_export();
//     //             Ok(zones)
//     //         }
//     //         Err(err) => Err(JsValue::from_str(&err.to_string())),
//     //     }
//     // }

//     // #[wasm_bindgen(js_name = zonesFromParent)]
//     // pub fn zones_from_parent(
//     //     &self,
//     //     depth: u8,
//     //     parent_zone_id: String,
//     //     densify: bool,
//     // ) -> Result<JsZones, JsValue> {
//     //     match self.inner.zones_from_parent(depth, parent_zone_id, densify) {
//     //         Ok(z) => {
//     //             let zones = z.to_export();
//     //             Ok(zones)
//     //         }
//     //         Err(err) => Err(JsValue::from_str(&err.to_string())),
//     //     }
//     // }

//     // #[wasm_bindgen(js_name = zoneFromId)]
//     // pub fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<JsZones, JsValue> {
//     //     match self.inner.zone_from_id(zone_id, densify) {
//     //         Ok(z) => {
//     //             let zones = z.to_export();
//     //             Ok(zones)
//     //         }
//     //         Err(err) => Err(JsValue::from_str(&err.to_string())),
//     //     }
//     // }
// }
