// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

//use crate::adapters::healpix::common::{refinement_level_to_h3_resolution, to_zones};
use crate::adapters::healpixgeo::healpixgeo::HEALPixGeoAdapter;
use crate::api::{DggrsApi, DggrsApiConfig};
use crate::error::DggrsError;
use crate::error::healpixgeo::HEALPixGeoError;
use crate::models::common::{DggrsUid, RefinementLevel, RelativeDepth, ZoneId, Zones};
use geo::{Point, Rect};
use healpix_geo::healpix;

pub struct HEALPixImpl {
    id: DggrsUid,
    adapter: HEALPixGeoAdapter,
}

impl HEALPixImpl {
    pub fn new() -> Self {
        Self {
            id: DggrsUid::HEALPIX,
            adapter: HEALPixGeoAdapter::new(),
        }
    }
}

impl Default for HEALPixImpl {
    fn default() -> Self {
        Self {
            id: DggrsUid::HEALPIX,
            adapter: HEALPixGeoAdapter::default(),
        }
    }
}

impl DggrsApi for HEALPixImpl {
    fn zones_from_bbox(
        &self,
        refinement_level: RefinementLevel,
        bbox: Option<Rect<f64>>,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        todo!();
        //healpix::
    }

    /// Get zones for a geo::Point.
    fn zone_from_point(
        &self,
        refinement_level: RefinementLevel,
        point: Point, // NOTE:Consider accepting a vector of Points.
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        todo!();
    }
    /// Get zones based on a parent ZoneID.
    fn zones_from_parent(
        &self,
        relative_depth: RelativeDepth,
        parent_zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        todo!();
    }

    /// Get a zone based on a ZoneID
    fn zone_from_id(
        &self,
        zone_id: ZoneId,
        config: Option<DggrsApiConfig>,
    ) -> Result<Zones, DggrsError> {
        todo!();
    }

    /// Get the minimum refinement level of a DGGRS
    fn min_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        todo!();
    }

    /// Get the maximum refinment level of a DGGRS
    fn max_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        todo!();
    }

    /// Get the default refinement level of a DGGRS
    fn default_refinement_level(&self) -> Result<RefinementLevel, DggrsError> {
        todo!();
    }

    /// Get the  max relative depth of a DGGRS
    fn max_relative_depth(&self) -> Result<RelativeDepth, DggrsError> {
        todo!();
    }

    /// Get the  default relative depth of a DGGRS
    fn default_relative_depth(&self) -> Result<RelativeDepth, DggrsError> {
        todo!();
    }
}
