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

    let position = Point::new(-9.222154, 38.695125);
    let projection = Vgc;
    let icosahedron = new();
    let barycentric_coords = projection.geo_to_bary(vec![position], Some(&icosahedron));

    println!("{:?}", barycentric_coords);

    // let position = barycentric_coords.iter().map(|f| f.coords).collect();
    // let geo_coords = projection.bary_to_geo(position);

    // println!("{:?}", geo_coords);
}
