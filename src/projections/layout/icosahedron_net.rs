// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use super::traits::Layout;
use geo::Coord;

#[derive(Default, Debug)]
pub struct IcosahedronNet {}

impl Layout for IcosahedronNet {
    fn face_center(&self, p: [(u8, u8); 3]) -> Coord {
        Coord {
            x: f64::from((p[0].0 + p[1].0 + p[2].0) / 3),
            y: f64::from((p[0].1 + p[1].1 + p[2].1) / 3),
        }
    }

    fn grid_size(&self) -> (usize, usize) {
        todo!()
    }
    fn vertices(&self) -> Vec<[(u8, u8); 3]> {
        todo!()
    }
}