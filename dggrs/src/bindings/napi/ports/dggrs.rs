// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use std::path::PathBuf;

use napi::Error;

use crate::{
    adapters::dggrid::{igeo7::Igeo7Impl, isea3h::Isea3hImpl},
    bindings::napi::models::common::JsZones,
    error::port::PortError,
    models::common::Zones,
    ports::dggrs::DggrsPort,
};

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

impl DggrsPort for DggrsPortEnum {
    fn zones_from_bbox(
        &self,
        depth: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> Result<Zones, PortError> {
        match self {
            DggrsPortEnum::Isea3h(port) => port.zones_from_bbox(depth, densify, bbox),
            DggrsPortEnum::Igeo7(port) => port.zones_from_bbox(depth, densify, bbox),
        }
    }

    fn zone_from_point(
        &self,
        depth: u8,
        point: geo::Point,
        densify: bool,
    ) -> Result<Zones, PortError> {
        match self {
            DggrsPortEnum::Isea3h(port) => port.zone_from_point(depth, point, densify),
            DggrsPortEnum::Igeo7(port) => port.zone_from_point(depth, point, densify),
        }
    }

    fn zones_from_parent(
        &self,
        depth: u8,
        parent_zone_id: String,
        densify: bool,
    ) -> Result<Zones, PortError> {
        match self {
            DggrsPortEnum::Isea3h(port) => port.zones_from_parent(depth, parent_zone_id, densify),
            DggrsPortEnum::Igeo7(port) => port.zones_from_parent(depth, parent_zone_id, densify),
        }
    }

    fn zone_from_id(&self, zone_id: String, densify: bool) -> Result<Zones, PortError> {
        match self {
            DggrsPortEnum::Isea3h(port) => port.zone_from_id(zone_id, densify),
            DggrsPortEnum::Igeo7(port) => port.zone_from_id(zone_id, densify),
        }
    }
    // forward the rest...
}

#[napi]
impl Dggrs {
    #[napi(constructor)]
    pub fn new(dggrs: String) -> Dggrs {
        Dggrs {
            inner: match dggrs.as_str() {
                "isea3h" => DggrsPortEnum::Isea3h(Isea3hImpl::new(
                    PathBuf::from("dggrid"),
                    PathBuf::from(""),
                )),
                "igeo7" => {
                    DggrsPortEnum::Igeo7(Igeo7Impl::new(PathBuf::from("dggrid"), PathBuf::from("")))
                }
                _ => panic!("Type a valid DGGRS"),
            },
        }
    }

    #[napi(js_name = zonesFromBbox)]
    pub fn zones_from_bbox(
        &self,
        depth: u8,
        densify: bool,
        bbox: Option<Vec<Vec<f64>>>,
    ) -> napi::Result<JsZones> {
        let zones = self
            .inner
            .zones_from_bbox(depth, densify, bbox)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(zones.to_export())
    }

    #[napi(js_name = zoneFromPoint)]
    pub fn zone_from_point(
        &self,
        depth: u8,
        point: Vec<f64>,
        densify: bool,
    ) -> napi::Result<JsZones> {
        let geo_pt = geo::Point::new(point[0], point[1]);
        let zones = self
            .inner
            .zone_from_point(depth, geo_pt, densify)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(zones.to_export())
    }

    #[napi(js_name = zonesFromParent)]
    pub fn zones_from_parent(
        &self,
        depth: u8,
        parent_zone_id: String,
        densify: bool,
    ) -> napi::Result<JsZones> {
        let zones = self
            .inner
            .zones_from_parent(depth, parent_zone_id, densify)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(zones.to_export())
    }

    #[napi(js_name = zoneFromId)]
    pub fn zone_from_id(&self, zone_id: String, densify: bool) -> napi::Result<JsZones> {
        let zones = self
            .inner
            .zone_from_id(zone_id, densify)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(zones.to_export())
    }
}
