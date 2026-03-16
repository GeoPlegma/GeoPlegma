// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
// Modified by João Manuel (joao.manuel@geoinsight.ai)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use std::vec;

use geo::Point;
use gp_proj::{
    constants::KarneyCoefficients,
    projections::{
        polyhedron::{
            icosahedron::new,
            spherical_geometry::{self, spherical_triangle_area},
        },
        projections::{
            traits::{ForwardBary, Projection},
            vgc::Vgc,
        },
    },
    utils::shape::triangle3d_to_2d,
};

pub fn main() -> () {
    println!(
        "Basic example for gp-proj. Convert geographic coordinates to barycentric coordinates, and vice-versa."
    );

    let points = [
        Point::new(-9.222154, 38.695125),
        Point::new(-138.97503, 47.7022),
        Point::new(99.72721, 25.82577),
        Point::new(-64.10552, 12.89276),
        Point::new(-128.28185, -50.60992),
        Point::new(-70.47681, -0.81784),
        Point::new(152.44705, -21.59114),
        Point::new(66.665798, -77.717034),
        Point::new(63.501735, 80.099071),
        Point::new(0.0, 45.0),
        // Point::new(20.0, 50.0),
        // Point::new(40.0, 55.0), 
        Point::new(30.0, 30.0),
    ];

    let projection = Vgc;
    let icosahedron = new();

    let coords = projection.geo_to_face(points.to_vec(), Some(&icosahedron));

    // let coef = Vgc::fourier_coefficients(KarneyCoefficients::GEODETIC_TO_AUTHALIC);
    // let coords_cart = projection.geo_to_cartesian(points.to_vec(), Some(&icosahedron), None);

    for (i, c) in coords.iter().enumerate() {
        println!("----------------------------------");
        // println!("Longitude: {}, Latitude: {}", points[i].x(), points[i].y());
        // println!("Face: {}", c.face);
        println!(
            "{} Barycentric ({:?},{:?},{:?})",
            i+1, c.coords.x, c.coords.y, c.coords.z
        );
        //  println!(
        //     "Cartesian coordinates: ({:?},{:?})",
        //     coords_cart[i].coords.x, coords_cart[i].coords.y
        // );
    }
    // println!(
    //     "Basic example for gp-proj. Convert geographic coordinates to cartesian coordinates, and vice-versa."
    // // );
    let coef = Vgc::fourier_coefficients(KarneyCoefficients::GEODETIC_TO_AUTHALIC);

    // // // let mut all: Vec<Point> = [].to_vec();
    let polyhedron = new();
    // for lat in [10.0, 30.0, 50.0, 70.0] {
    // // for lat in [0.0, 20.0, 40.0] {
    // //     for lon in [45.0, 50.0,55.0] {
    //     for lon in [-120.0, -60.0, 0.0, 60.0, 120.0] {
    //          let p1 = Point::new(lon, lat);
    //         let p2 = Point::new(lon + 1.0, lat);
    //         let p3 = Point::new(lon + 1.0, lat + 1.0);
    //         let p4 = Point::new(lon, lat + 1.0);

    //         let bary_results = projection.geo_to_face(vec![p1, p2, p3, p4], Some(&polyhedron));

    //         // Skip if not all on same face
    //         if bary_results.len() != 4 ||
    //            bary_results[0].face != bary_results[1].face ||
    //            bary_results[0].face != bary_results[2].face ||
    //            bary_results[0].face != bary_results[3].face {
    //             continue;
    //         }

    //         let face = bary_results[0].face;

    //         // Convert barycentric to 2D Cartesian for area calculation
    //         let face_verts = polyhedron.face_vertices(face).unwrap();
    //         let face_edges = polyhedron.face_arc_lengths(face).unwrap();
    //         let is_upward = face % 2 == 0;
    //         let face_area = spherical_triangle_area([face_verts[0], face_verts[1], face_verts[2]]).unwrap();
    //         let face_2d = triangle3d_to_2d(face_edges[0], face_edges[1], face_edges[2], is_upward, face_area);

    //         let r = 6371007.181;

    //         let to_xy = |bary: &ForwardBary| -> (f64, f64) {
    //             let u = bary.coords.x;
    //             let v = bary.coords.y;
    //             let w = bary.coords.z;

    //             let x = (face_2d[0].0 * u + face_2d[1].0 * v + face_2d[2].0 * w) * r;
    //             let y = (face_2d[0].1 * u + face_2d[1].1 * v + face_2d[2].1 * w) * r;
    //             (x, y)
    //         };

    //         let coords: Vec<(f64, f64)> = bary_results.iter().map(to_xy).collect();

    //         // Authalic spherical area
    //         let lat0_auth = Vgc::lat_geodetic_to_authalic(lat.to_radians(), &coef).to_degrees();
    //         let lat1_auth = Vgc::lat_geodetic_to_authalic((lat + 1.0).to_radians(), &coef).to_degrees();
    //         let dlon_rad = 1.0_f64.to_radians();
    //         let spherical_area = r.powi(2) * dlon_rad *
    //             (lat1_auth.to_radians().sin() - lat0_auth.to_radians().sin()).abs();

    //         // Projected area
    //         let projected_area = 0.5 * (
    //             (coords[0].0 * coords[1].1 - coords[1].0 * coords[0].1) +
    //             (coords[1].0 * coords[2].1 - coords[2].0 * coords[1].1) +
    //             (coords[2].0 * coords[3].1 - coords[3].0 * coords[2].1) +
    //             (coords[3].0 * coords[0].1 - coords[0].0 * coords[3].1)
    //         ).abs();

    //         let ratio = projected_area / spherical_area;

    //         println!("lat={}, lon={}, face={}, ratio={:.4}", lat, lon, face, ratio);
    //     }
    // }

    // let coords = projection.geo_to_face(all.to_vec(), Some(&icosahedron));
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
