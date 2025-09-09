// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::models::common::{
    DggrsImplementation, DggrsName, DggrsSpec, DggrsUid, RefinementLevel, RelativeDepth,
};
use geo::{Coord, Rect};

pub fn whole_earth_bbox() -> Rect<f64> {
    Rect::new(
        Coord {
            x: -180.0,
            y: -90.0,
        },
        Coord { x: 180.0, y: 90.0 },
    )
}

pub const DGGRS_SPECS: [DggrsSpec; 9] = [
    DggrsSpec {
        id: DggrsUid::ISEA3HDGGRID,
        name: DggrsName::ISEA3H,
        tool: DggrsImplementation::DGGRID,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(36),
        default_refinement_level: RefinementLevel::new_const(3),
        max_relative_depth: RelativeDepth::new_const(8),
        default_relative_depth: RelativeDepth::new_const(5),
    },
    DggrsSpec {
        id: DggrsUid::IGEO7,
        name: DggrsName::IGEO7,
        tool: DggrsImplementation::DGGRID,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(20),
        default_refinement_level: RefinementLevel::new_const(2),
        max_relative_depth: RelativeDepth::new_const(5),
        default_relative_depth: RelativeDepth::new_const(3),
    },
    DggrsSpec {
        id: DggrsUid::H3,
        name: DggrsName::H3,
        tool: DggrsImplementation::H3O,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(16),
        default_refinement_level: RefinementLevel::new_const(2),
        max_relative_depth: RelativeDepth::new_const(6),
        default_relative_depth: RelativeDepth::new_const(4),
    },
    DggrsSpec {
        id: DggrsUid::ISEA3HDGGAL,
        name: DggrsName::ISEA3H,
        tool: DggrsImplementation::DGGAL,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(33),
        default_refinement_level: RefinementLevel::new_const(3),
        max_relative_depth: RelativeDepth::new_const(11),
        default_relative_depth: RelativeDepth::new_const(8),
    },
    DggrsSpec {
        id: DggrsUid::IVEA3H,
        name: DggrsName::IVEA3H,
        tool: DggrsImplementation::DGGAL,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(33),
        default_refinement_level: RefinementLevel::new_const(3),
        max_relative_depth: RelativeDepth::new_const(11),
        default_relative_depth: RelativeDepth::new_const(8),
    },
    DggrsSpec {
        id: DggrsUid::ISEA9R,
        name: DggrsName::ISEA9R,
        tool: DggrsImplementation::DGGAL,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(16),
        default_refinement_level: RefinementLevel::new_const(2),
        max_relative_depth: RelativeDepth::new_const(8),
        default_relative_depth: RelativeDepth::new_const(2),
    },
    DggrsSpec {
        id: DggrsUid::IVEA9R,
        name: DggrsName::IVEA9R,
        tool: DggrsImplementation::DGGAL,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(16),
        default_refinement_level: RefinementLevel::new_const(2),
        max_relative_depth: RelativeDepth::new_const(6),
        default_relative_depth: RelativeDepth::new_const(4),
    },
    DggrsSpec {
        id: DggrsUid::RTEA3H,
        name: DggrsName::RTEA3H,
        tool: DggrsImplementation::DGGAL,
        title: "",
        description: "",
        uri: "",
        crs: "",
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(33),
        default_refinement_level: RefinementLevel::new_const(2),
        max_relative_depth: RelativeDepth::new_const(11),
        default_relative_depth: RelativeDepth::new_const(8),
    },
    DggrsSpec {
        id: DggrsUid::RTEA9R,
        name: DggrsName::RTEA9R,
        title: "",
        description: "",
        uri: "",
        crs: "",
        tool: DggrsImplementation::DGGAL,
        min_refinement_level: RefinementLevel::new_const(0),
        max_refinement_level: RefinementLevel::new_const(16),
        default_refinement_level: RefinementLevel::new_const(2),
        max_relative_depth: RelativeDepth::new_const(6),
        default_relative_depth: RelativeDepth::new_const(4),
    },
];
