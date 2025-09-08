// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use geo::Point;
use gp_proj::{projections::{layout::icosahedron_net::IcosahedronNet, polyhedron::icosahedron::new, projections::{traits::Projection, vgc::Vgc}}, Vector3D};

pub fn main() -> () {
    println!("Basic example for gp-proj");

    let position = Point::new(-9.222154, 38.695125);
    let projection = Vgc;
    let icosahedron = new();
    let result = projection.forward(vec![position], Some(&icosahedron), &IcosahedronNet {});

    println!("{:?}", result);
}
