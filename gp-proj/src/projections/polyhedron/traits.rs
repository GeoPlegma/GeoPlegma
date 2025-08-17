// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Modified by Sunayana Ghosh (sunayanag@gmail.com)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms


#[derive(Debug, Clone)]
pub enum Face {
    Triangle([usize; 3]),
    Quad([usize; 4]),
    Pentagon([usize; 5]),
    Hexagon([usize; 6]),
    Polygon(Vec<usize>), // for rare or irregular faces
}

impl Face {
    pub fn indices(&self) -> &[usize] {
        match self {
            Face::Triangle(v) => v,
            Face::Quad(v) => v,
            Face::Pentagon(v) => v,
            Face::Hexagon(v) => v,
            Face::Polygon(v) => v,
        }
    }
}

#[derive(Default)]
pub struct ArcLengths {
    pub ab: f64,
    pub bc: f64,
    pub ac: f64,
    pub ap: f64,
    pub bp: f64,
    pub cp: f64,
}
