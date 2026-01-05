// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use std::{str::FromStr, sync::Arc};

use api::{
  adapters::{
    dggal::grids::DggalImpl,
    dggrid::{igeo7::Igeo7Impl, isea3h::Isea3hImpl},
  },
  api::{DggrsApi, DggrsApiConfig},
  factory,
  models::common::{DggrsUid, HexString, RefinementLevel, RelativeDepth},
};
use geo::{Coord, Rect};
use napi::{Either, Error};

use crate::models::common::{JsZones, ZonesWrapper};

use napi_derive::napi;

#[napi]
pub struct Dggrs {
  inner: Arc<dyn DggrsApi>,
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

#[napi]
impl Dggrs {
  #[napi(constructor)]
  pub fn new(dggrs: String) -> Dggrs {
    let dggrs_uid = DggrsUid::from_str(&dggrs).expect("Invalid DGGRS UID");

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
    let generator = Dggrs::new("ISEA3HDGGAL".to_owned());
    let rl = RefinementLevel::new(1).unwrap();
    let bbox = Rect::new([-77.0, 39.0], [-76.0, 40.0]);
    let result = generator
      .inner
      .zones_from_bbox(rl, Some(bbox), None)
      .unwrap();
    
    assert_eq!(
      result.zones.len(),
      1,
      "{:?}: zones_from_bbox returned wrong result",
      result.zones
    );
  }
}
