// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
// Modified by João Manuel (joao.manuel@geoinsight.ai)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use gp_proj::projections::{
    polyhedron::icosahedron::new,
    projections::{traits::Projection, vgc::Vgc},
};

pub fn main() -> () {
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let edge1 = Point::new(0.0, 45.0);
    let edge2 = Point::new(30.0, 30.0);
    let random_point = Point::new(-9.49420, 38.68499);

    let projection = Vgc;
    let icosahedron = new();
    let coords = projection.geo_to_face(vec![edge1, edge2, random_point], Some(&icosahedron));

    println!("{:?}", coords);
}
