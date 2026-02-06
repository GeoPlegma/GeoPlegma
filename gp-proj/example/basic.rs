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
    polyhedron::{icosahedron::new, spherical_geometry},
    projections::{traits::Projection, vgc::Vgc},
};

pub fn main() -> () {
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let points = [
        Point::new(0.0, 45.0),
        Point::new(30.0, 30.0),
        Point::new(-9.49420, 38.68499),
    ];

    let projection = Vgc;
    let icosahedron = new();

    let coords = projection.geo_to_face(points.to_vec(), Some(&icosahedron));

    for (i, c) in coords.iter().enumerate() {
        println!("Latitude: {}, Longitude: {}", points[i].x(), points[i].y());
        println!("Face: {}", c.face);
        println!("Barycentric coordinates: {:?}", c.coords);
    }
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to cartesian coordinates, and vice-versa."
    );

    let points1 = vec![
        Point::new(0.0, 45.0),
        Point::new(30.0, 30.0),
        Point::new(-9.49420, 38.68499),
    ];
    let coords = projection.geo_to_cartesian(points1, Some(&icosahedron), None);
    for (i, c) in coords.iter().enumerate() {
        println!("Latitude: {}, Longitude: {}", points[i].x(), points[i].y());
        println!("Face: {}", c.face);
        println!(
            "Cartesian coordinates (origin on right most corner of face): {:?}",
            c.coords
        );
    }

    // println!("{:?}", coords);

    // let distortion = projection.compute_distortion(38.68499, -9.49420, &icosahedron);
    // println!("h: {} (expected: 0.7580403)", distortion.h);
    // println!("k: {} (expected: 1.333174)", distortion.k);
    // println!(
    //     "Angular deformation: {}° (expected: 33.045°)",
    //     distortion.angular_deformation
    // );
    // println!("Areal scale: {} (expected: ~1.0)", distortion.areal_scale);
}
