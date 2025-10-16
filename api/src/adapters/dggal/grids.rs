// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::dggal::common::{bbox_to_geoextent, to_geo_point, to_zones};
use crate::adapters::dggal::context::GLOBAL_DGGAL;
use crate::api::{DggrsApi, DggrsApiConfig};
use crate::constants::whole_earth_bbox;
use crate::error::DggrsError;
use crate::error::dggal::DggalError;
use crate::models::common::{DggrsName, DggrsUid, RefinementLevel, RelativeDepth, ZoneId, Zones};
use dggal::DGGRS;
use dggal_rust::dggal;
use geo::{Point, Rect};

pub struct DggalImpl {
    pub id: DggrsUid,
}

impl DggalImpl {
    pub fn new(id: DggrsUid) -> Self {
        Self { id }
    }

    #[inline]
    fn grid_name(&self) -> DggrsName {
        self.id.spec().name
    }

    fn get_dggrs(&self) -> Result<DGGRS, DggalError> {
        let dggal = GLOBAL_DGGAL.lock().map_err(|_| DggalError::LockFailure)?;
        DGGRS::new(&*dggal, &self.grid_name().to_string()).map_err(|_| DggalError::UnknownGrid {
            grid_name: self.grid_name().to_string(),
        })
    }
}

impl DggrsApi for DggalImpl {
    fn zones_from_bbox(
        &self,
        refinement_level: RefinementLevel,
        bbox: Option<Rect<f64>>,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        if refinement_level > self.max_refinement_level()? {
            return Err(DggrsError::RefinementLevelLimitReached {
                grid_name: self.grid_name().to_string(),
                requested: refinement_level,
                maximum: self.max_refinement_level()?,
            });
        };

        let geo_extent = if let Some(b) = bbox {
            bbox_to_geoextent(&b)
        } else {
            bbox_to_geoextent(&whole_earth_bbox())
        };

        let dggrs = self.get_dggrs()?;

        let zones = dggrs.listZones(i32::from(refinement_level), &geo_extent);
        Ok(to_zones(dggrs, zones, cfg)?)
    }
    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let dggrs = self.get_dggrs()?;
        let zone = dggrs.getZoneFromWGS84Centroid(refinement_level.get(), &to_geo_point(point));
        let zones = vec![zone];
        Ok(to_zones(dggrs, zones, cfg)?)
    }
    fn zones_from_parent(
        &self,
        relative_depth: RelativeDepth,
        parent_zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();

        let dggrs = self.get_dggrs()?;

        // Check if ParentZoneId is Int
        let parent_zone_u64 = match &parent_zone_id {
            ZoneId::IntId(id) => *id,
            ZoneId::StrId(s) => dggrs.getZoneFromTextID(s),
            ZoneId::HexId(h) => dggrs.getZoneFromTextID(&h.to_string()),
        };

        if relative_depth > self.max_relative_depth()? {
            return Err(DggrsError::RelativeDepthLimitReached {
                grid_name: self.grid_name().to_string(),
                requested: relative_depth,
                maximum: self.max_relative_depth()?,
            });
        };

        let target_level =
            RefinementLevel::new(dggrs.getZoneLevel(parent_zone_u64))?.add(relative_depth)?;

        if target_level > self.max_refinement_level()? {
            return Err(DggrsError::RefinementLevelPlusRelativeDepthLimitReached {
                grid_name: self.grid_name().to_string(),
                requested: relative_depth,
                maximum: self.max_refinement_level()?,
            });
        };

        let zones = dggrs.getSubZones(parent_zone_u64, i32::from(relative_depth));

        Ok(to_zones(dggrs, zones, cfg)?)
    }
    fn zone_from_id(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();

        let dggrs = self.get_dggrs()?;

        // Check if ZoneId is Int
        let zone_u64 = match &zone_id {
            ZoneId::IntId(id) => *id,
            ZoneId::StrId(s) => dggrs.getZoneFromTextID(s),
            ZoneId::HexId(h) => dggrs.getZoneFromTextID(&h.to_string()),
        };

        let zones = vec![zone_u64];

        Ok(to_zones(dggrs, zones, cfg)?)
    }

    fn min_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        Ok(self.id.spec().min_refinement_level)
    }

    fn max_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        Ok(self.id.spec().max_refinement_level)
    }

    fn default_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        Ok(self.id.spec().default_refinement_level)
    }

    fn max_relative_depth(&self) -> Result<RelativeDepth, DggrsError> {
        Ok(self.id.spec().max_relative_depth)
    }

    fn default_relative_depth(&self) -> Result<RelativeDepth, DggrsError> {
        Ok(self.id.spec().default_relative_depth)
    }
}
