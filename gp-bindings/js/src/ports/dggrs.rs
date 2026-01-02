// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use std::{path::PathBuf, str::FromStr, sync::Arc};

use api::{
  adapters::{
    dggal::grids::DggalImpl,
    dggrid::{igeo7::Igeo7Impl, isea3h::Isea3hImpl},
  },
  api::{DggrsApi, DggrsApiConfig},
  error::DggrsError,
  factory,
  models::common::{DggrsUid, HexString, RefinementLevel, RelativeDepth, ZoneId, Zones},
};
use geo::{Coord, Point, Rect};
use napi::{Either, Error};

use crate::models::common::{JsZones, ZonesWrapper};

use napi_derive::napi;

#[napi]
pub struct Dggrs {
  inner: Arc<dyn DggrsApi>,
}

pub enum DggrsApiEnum {
  Isea3h(Isea3hImpl),
  Igeo7(Igeo7Impl),
  Dggal(DggalImpl),
  // ... future implementors
}

#[napi(object)]
pub struct Config {
  pub region: bool,
  pub center: bool,
  pub vertex_count: bool,
  pub children: bool,
  pub neighbors: bool,
  pub area_sqm: bool,
  pub densify: bool, // TODO:: this is the switch to generate densified gemetry, which is actually not needed for H3 due to the Gnomic projection.
}

#[napi]
impl Default for Config {
  fn default() -> Self {
    Self {
      region: true,
      center: true,
      vertex_count: true,
      children: true,
      neighbors: true,
      area_sqm: true,
      densify: true,
    }
  }
}

#[napi]
pub fn default_config() -> Config {
  Config {
    region: true,
    center: true,
    vertex_count: true,
    children: true,
    neighbors: true,
    area_sqm: true,
    densify: true,
  }
}

// impl DggrsApi for DggrsApiEnum {
//   fn zones_from_bbox(
//     &self,
//     refinement_level: RefinementLevel,
//     bbox: Option<Rect<f64>>,
//     config: Option<DggrsApiConfig>,
//   ) -> Result<Zones, DggrsError> {
//     match self {
//       DggrsApiEnum::Isea3h(port) => port.zones_from_bbox(refinement_level, bbox, config),
//       DggrsApiEnum::Igeo7(port) => port.zones_from_bbox(refinement_level, bbox, config),
//       DggrsApiEnum::Dggal(port) => port.zones_from_bbox(refinement_level, bbox, config),
//     }
//   }

//   fn zone_from_point(
//     &self,
//     refinement_level: RefinementLevel,
//     point: Point, // NOTE:Consider accepting a vector of Points.
//     config: Option<DggrsApiConfig>,
//   ) -> Result<Zones, DggrsError> {
//     match self {
//       DggrsApiEnum::Isea3h(port) => port.zone_from_point(refinement_level, point, config),
//       DggrsApiEnum::Igeo7(port) => port.zone_from_point(refinement_level, point, config),
//       DggrsApiEnum::Dggal(port) => port.zone_from_point(refinement_level, point, config),
//     }
//   }

//   fn zones_from_parent(
//     &self,
//     relative_depth: RelativeDepth,
//     parent_zone_id: ZoneId,
//     config: Option<DggrsApiConfig>,
//   ) -> Result<Zones, DggrsError> {
//     match self {
//       DggrsApiEnum::Isea3h(port) => port.zones_from_parent(relative_depth, parent_zone_id, config),
//       DggrsApiEnum::Igeo7(port) => port.zones_from_parent(relative_depth, parent_zone_id, config),
//       DggrsApiEnum::Dggal(port) => port.zones_from_parent(relative_depth, parent_zone_id, config),
//     }
//   }

//   fn zone_from_id(
//     &self,
//     zone_id: ZoneId,
//     config: Option<DggrsApiConfig>,
//   ) -> Result<Zones, DggrsError> {
//     match self {
//       DggrsApiEnum::Isea3h(port) => port.zone_from_id(zone_id, config),
//       DggrsApiEnum::Igeo7(port) => port.zone_from_id(zone_id, config),
//       DggrsApiEnum::Dggal(port) => port.zone_from_id(zone_id, config),
//     }
//   }

//   fn min_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
//     todo!()
//   }

//   fn max_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
//     todo!()
//   }

//   fn default_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
//     todo!()
//   }

//   fn max_relative_depth(&self) -> Result<api::models::common::RelativeDepth, DggrsError> {
//     todo!()
//   }

//   fn default_relative_depth(&self) -> Result<api::models::common::RelativeDepth, DggrsError> {
//     todo!()
//   }
//   // forward the rest...
// }

#[napi]
impl Dggrs {
  #[napi(constructor)]
  pub fn new(dggrs: String) -> Dggrs {
    let dggrs_uid = DggrsUid::from_str(&dggrs).expect("Invalid DGGRS UID");

    // Dggrs {
    //   inner: match dggrs.as_str() {
    //     "isea3h" => {
    //       DggrsApiEnum::Isea3h(Isea3hImpl::new(PathBuf::from("dggrid"), PathBuf::from("")))
    //     }
    //     "igeo7" => DggrsApiEnum::Igeo7(Igeo7Impl::new(PathBuf::from("dggrid"), PathBuf::from(""))),
    //     "dggal" => DggrsApiEnum::Dggal(DggalImpl::new(dggrs_uid)),
    //     _ => panic!("Type a valid DGGRS"),
    //   },
    // }
    Dggrs {
      inner: factory::get(dggrs_uid).expect("msg"),
    }
  }

  #[napi(js_name = zonesFromBbox)]
  pub fn zones_from_bbox(
    &self,
    refinement_level: i32,
    bbox: Option<Vec<Vec<f64>>>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let refinement_level_ = RefinementLevel::new(refinement_level).unwrap();

    let bbox_: Option<Rect> = match bbox {
      Some(b) => Some(Rect::new(
        Coord {
          x: b[0][0],
          y: b[0][1],
        },
        Coord {
          x: b[1][0],
          y: b[1][1],
        },
      )),
      _ => None,
    };

    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zones_from_bbox(refinement_level_, bbox_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }

  #[napi(js_name = zoneFromPoint)]
  pub fn zone_from_point(
    &self,
    refinement_level: i32,
    point: Option<Vec<f64>>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let refinement_level_ = RefinementLevel::new(refinement_level).unwrap();
    let point_ = point.unwrap();
    let geo_pt = geo::Point::new(point_[0], point_[1]);

    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zone_from_point(refinement_level_, geo_pt, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };
    Ok(zones.to_export())
  }

  #[napi(js_name = zonesFromParent)]
  pub fn zones_from_parent(
    &self,
    relative_depth: i32,
    parent_zone_id: Either<String, i64>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let relative_depth_ = RelativeDepth::new(relative_depth).unwrap();
    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let parent_zone_id_ = match parent_zone_id {
      Either::B(num) => api::models::common::ZoneId::IntId(num.try_into().unwrap()),

      Either::A(s) => {
        if is_zone_hex_id(&s) {
          api::models::common::ZoneId::HexId(HexString::new(&s).unwrap())
        } else {
          api::models::common::ZoneId::StrId(s)
        }
      }
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zones_from_parent(relative_depth_, parent_zone_id_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }

  #[napi(js_name = zoneFromId)]
  pub fn zone_from_id(
    &self,
    zone_id: Either<String, i64>,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsApiConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };

    let zone_id_ = match zone_id {
      Either::B(num) => api::models::common::ZoneId::IntId(num.try_into().unwrap()),

      Either::A(s) => {
        if is_zone_hex_id(&s) {
          api::models::common::ZoneId::HexId(HexString::new(&s).unwrap())
        } else {
          api::models::common::ZoneId::StrId(s)
        }
      }
    };

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zone_from_id(zone_id_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }
}
fn is_zone_hex_id(s: &str) -> bool {
  s.len() == 16 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_zone() {

   assert_eq!(is_zone_hex_id("B4-8-B"), true)
  }
}
