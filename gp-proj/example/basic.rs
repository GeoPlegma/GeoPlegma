// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
// Modified by João Manuel (joao.manuel@geoinsight.ai)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use gp_proj::{
    Vector3D,
    projections::{
        polyhedron::icosahedron::new,
        projections::{traits::Projection, vgc::Vgc},
    },
};

pub fn main() -> () {
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let p1 = Point::new(-9.222154, 38.695125);
    let p2 = Point::new(-138.97503, 47.7022);
    let p3 = Point::new(99.72721, 25.82577);
    let p4 = Point::new(-64.10552, 12.89276);
    let p5 = Point::new(-128.28185, -50.60992);
    let p6 = Point::new(-70.47681, -0.81784);
    let p7 = Point::new(152.44705, -21.59114);
    let p8 = Point::new(66.665798, -77.717034);
    let p9 = Point::new(63.501735, 80.099071);

    let projection = Vgc;
    let icosahedron = new();
    let barycentric_coords =
        projection.geo_to_bary(vec![p1], Some(&icosahedron));

    println!("{:?}", barycentric_coords);

    // let position = barycentric_coords.iter().map(|f| f.coords).collect();
    // let geo_coords = projection.bary_to_geo(position);

    // println!("{:?}", geo_coords);
}
