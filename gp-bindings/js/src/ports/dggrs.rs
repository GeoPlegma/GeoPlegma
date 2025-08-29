// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use std::path::PathBuf;

use dggrs::{
  adapters::dggrid::{igeo7::Igeo7Impl, isea3h::Isea3hImpl},
  error::port::GeoPlegmaError,
  models::common::{HexString, RefinementLevel, RelativeDepth, ZoneId, Zones},
  ports::dggrs::{DggrsPort, DggrsPortConfig},
};
use geo::{Coord, Point, Rect};
use napi::Error;

use crate::models::common::{JsZones, ZonesWrapper};

use napi_derive::napi;

#[napi]
pub struct Dggrs {
  inner: DggrsPortEnum,
}

pub enum DggrsPortEnum {
  Isea3h(Isea3hImpl),
  Igeo7(Igeo7Impl),
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

impl DggrsPort for DggrsPortEnum {
  fn zones_from_bbox(
    &self,
    refinement_level: RefinementLevel,
    bbox: Option<Rect<f64>>,
    config: Option<DggrsPortConfig>,
  ) -> Result<Zones, GeoPlegmaError> {
    match self {
      DggrsPortEnum::Isea3h(port) => port.zones_from_bbox(refinement_level, bbox, config),
      DggrsPortEnum::Igeo7(port) => port.zones_from_bbox(refinement_level, bbox, config),
    }
  }

  fn zone_from_point(
    &self,
    refinement_level: RefinementLevel,
    point: Point, // NOTE:Consider accepting a vector of Points.
    config: Option<DggrsPortConfig>,
  ) -> Result<Zones, GeoPlegmaError> {
    match self {
      DggrsPortEnum::Isea3h(port) => port.zone_from_point(refinement_level, point, config),
      DggrsPortEnum::Igeo7(port) => port.zone_from_point(refinement_level, point, config),
    }
  }

  fn zones_from_parent(
    &self,
    relative_depth: RelativeDepth,
    parent_zone_id: ZoneId,
    config: Option<DggrsPortConfig>,
  ) -> Result<Zones, GeoPlegmaError> {
    match self {
      DggrsPortEnum::Isea3h(port) => port.zones_from_parent(relative_depth, parent_zone_id, config),
      DggrsPortEnum::Igeo7(port) => port.zones_from_parent(relative_depth, parent_zone_id, config),
    }
  }

  fn zone_from_id(
    &self,
    zone_id: ZoneId,
    config: Option<DggrsPortConfig>,
  ) -> Result<Zones, GeoPlegmaError> {
    match self {
      DggrsPortEnum::Isea3h(port) => port.zone_from_id(zone_id, config),
      DggrsPortEnum::Igeo7(port) => port.zone_from_id(zone_id, config),
    }
  }

  fn min_refinement_level(&self) -> Result<RefinementLevel, GeoPlegmaError> {
    todo!()
  }

  fn max_refinement_level(&self) -> Result<RefinementLevel, GeoPlegmaError> {
    todo!()
  }

  fn default_refinement_level(&self) -> Result<RefinementLevel, GeoPlegmaError> {
    todo!()
  }

  fn max_relative_depth(&self) -> Result<dggrs::models::common::RelativeDepth, GeoPlegmaError> {
    todo!()
  }

  fn default_relative_depth(&self) -> Result<dggrs::models::common::RelativeDepth, GeoPlegmaError> {
    todo!()
  }
  // forward the rest...
}

#[napi]
impl Dggrs {
  #[napi(constructor)]
  pub fn new(dggrs: String) -> Dggrs {
    Dggrs {
      inner: match dggrs.as_str() {
        "isea3h" => {
          DggrsPortEnum::Isea3h(Isea3hImpl::new(PathBuf::from("dggrid"), PathBuf::from("")))
        }
        "igeo7" => DggrsPortEnum::Igeo7(Igeo7Impl::new(PathBuf::from("dggrid"), PathBuf::from(""))),
        _ => panic!("Type a valid DGGRS"),
      },
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
    let config_ = DggrsPortConfig {
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
    let config_ = DggrsPortConfig {
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
    parent_zone_id: String,
    config: Option<Config>,
  ) -> napi::Result<JsZones> {
    let relative_depth_ = RelativeDepth::new(relative_depth).unwrap();
    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsPortConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };
    let parent_zone_id_ = ZoneId::HexId(HexString::new(&parent_zone_id).unwrap());

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zones_from_parent(relative_depth_, parent_zone_id_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }

  #[napi(js_name = zoneFromId)]
  pub fn zone_from_id(&self, zone_id: String, config: Option<Config>) -> napi::Result<JsZones> {
    let config_unwrap = config.unwrap_or_default();
    let config_ = DggrsPortConfig {
      region: config_unwrap.region,
      center: config_unwrap.center,
      vertex_count: config_unwrap.vertex_count,
      children: config_unwrap.children,
      neighbors: config_unwrap.neighbors,
      area_sqm: config_unwrap.area_sqm,
      densify: config_unwrap.densify,
    };
    let zone_id_ = ZoneId::HexId(HexString::new(&zone_id).unwrap());

    let zones = ZonesWrapper {
      inner: self
        .inner
        .zone_from_id(zone_id_, Some(config_))
        .map_err(|e| Error::from_reason(e.to_string()))?,
    };

    Ok(zones.to_export())
  }
}
