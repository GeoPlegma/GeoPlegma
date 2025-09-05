// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::h3o::common::{refinement_level_to_h3_resolution, to_zones};
use crate::adapters::h3o::h3o::H3oAdapter;
use crate::error::DggrsError;
use crate::error::h3o::H3oError;
use crate::models::common::{DggrsUid, RefinementLevel, RelativeDepth, ZoneId, Zones};
use crate::ports::dggrs::{DggrsPort, DggrsPortConfig};
use geo::{Point, Rect};
use h3o::geom::{ContainmentMode, TilerBuilder};
use h3o::{CellIndex, LatLng};
use std::str::FromStr;

pub struct H3Impl {
    id: DggrsUid,
    adapter: H3oAdapter,
}

impl H3Impl {
    pub fn new() -> Self {
        Self {
            id: DggrsUid::H3,
            adapter: H3oAdapter::new(),
        }
    }
}

impl Default for H3Impl {
    fn default() -> Self {
        Self {
            id: DggrsUid::H3,
            adapter: H3oAdapter::default(),
        }
    }
}

impl DggrsPort for H3Impl {
    fn zones_from_bbox(
        &self,
        refinement_level: RefinementLevel,
        bbox: Option<Rect<f64>>,
        config: Option<DggrsPortConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let h3o_zones: Vec<CellIndex>;

        let mut tiler =
            TilerBuilder::new(refinement_level_to_h3_resolution(RefinementLevel::new(2)?)?)
                .containment_mode(ContainmentMode::Covers)
                .build();

        if let Some(b) = bbox {
            // NOTE: adapt resolution dynamically based on bbox size & depth
            let _ = tiler.add(b.to_polygon());
            h3o_zones = tiler.into_coverage().collect::<Vec<_>>();
        } else {
            if refinement_level > self.default_refinement_level()? {
                return Err(DggrsError::RefinementLevelTooHigh(refinement_level));
            }
            h3o_zones = CellIndex::base_cells()
                .flat_map(|base| {
                    base.children(
                        refinement_level_to_h3_resolution(refinement_level)
                            .expect("Cannot translate to H3 Resolution"), // NOTE: expect() because flat_map does not understand Result?
                    )
                })
                .collect::<Vec<_>>();
        }
        Ok(to_zones(h3o_zones, cfg)?)
    }
    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point, // TODO: we should support multiple points at once.
        config: Option<DggrsPortConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let coord = LatLng::new(point.x(), point.y()).expect("valid coord");

        let h3o_zone = coord.to_cell(refinement_level_to_h3_resolution(refinement_level)?);

        Ok(to_zones(vec![h3o_zone], cfg)?)
    }
    fn zones_from_parent(
        &self,
        relative_depth: RelativeDepth,
        parent_zone_id: ZoneId,
        config: Option<DggrsPortConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let parent = CellIndex::from_str(&parent_zone_id.to_string()).map_err(|e| {
            DggrsError::H3o(H3oError::InvalidZoneID {
                zone_id: parent_zone_id.to_string(),
                source: e,
            })
        })?;

        let target_level = RefinementLevel::new(parent.resolution() as i32)?.add(relative_depth)?;

        if target_level > self.max_refinement_level()? {
            return Err(DggrsError::RefinementLevelPlusRelativeDepthLimitReached {
                grid_name: self.id.spec().name.to_string(),
                requested: relative_depth,
                maximum: self.max_refinement_level()?,
            });
        }

        let h3o_sub_zones: Vec<CellIndex> = parent
            .children(refinement_level_to_h3_resolution(target_level)?)
            .collect();

        Ok(to_zones(h3o_sub_zones, cfg)?)
    }
    fn zone_from_id(
        &self,
        zone_id: ZoneId, // ToDo: needs validation function
        config: Option<DggrsPortConfig>,
    ) -> Result<Zones, DggrsError> {
        let cfg = config.unwrap_or_default();
        let h3o_zone = CellIndex::from_str(&zone_id.to_string()).map_err(|e| {
            DggrsError::H3o(H3oError::InvalidZoneID {
                zone_id: zone_id.to_string(),
                source: e,
            })
        })?;

        Ok(to_zones(vec![h3o_zone], cfg)?)
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
