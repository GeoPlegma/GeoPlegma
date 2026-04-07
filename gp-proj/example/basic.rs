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

    let coords = projection.geo_to_cartesian(points.to_vec(), Some(&icosahedron), None);

    // let coef = Vgc::fourier_coefficients(KarneyCoefficients::GEODETIC_TO_AUTHALIC);
    // let coords_cart = projection.geo_to_cartesian(points.to_vec(), Some(&icosahedron), None);

    for (i, c) in coords.iter().enumerate() {
        println!("----------------------------------");
        // println!("Longitude: {}, Latitude: {}", points[i].x(), points[i].y());
        // println!("Face: {}", c.face);
        // println!(
        //     "{} Barycentric ({:?},{:?},{:?})",
        //     i+1, c.coords.x, c.coords.y, c.coords.z
        // );
        println!(
            "{} Cartesian coordinates: ({:?},{:?})",
            i + 1,
            coords[i].coords.x,
            coords[i].coords.y
        );
    }
    // println!(
    //     "Basic example for gp-proj. Convert geographic coordinates to cartesian coordinates, and vice-versa."
    // // );
    let coef = Vgc::fourier_coefficients(KarneyCoefficients::GEODETIC_TO_AUTHALIC);
    let r = 6371007.181;
    // // // let mut all: Vec<Point> = [].to_vec();
    let polyhedron = new();
    for lat in [10.0, 30.0, 50.0, 70.0] {
    // for lat in [0.0, 20.0, 40.0] {
    //     for lon in [45.0, 50.0,55.0] {
        for lon in [-120.0, -60.0, 0.0, 60.0, 120.0] {
            let p1 = Point::new(lon,       lat);
            let p2 = Point::new(lon + 1.0, lat);
            let p3 = Point::new(lon + 1.0, lat + 1.0);
            let p4 = Point::new(lon,       lat + 1.0);

            let proj = projection.geo_to_cartesian(
                vec![p1, p2, p3, p4],
                Some(&icosahedron), None
            );

            if proj.len() < 4 {
                println!("lat={lat}, lon={lon} - SKIPPED (empty result)");
                continue;
            }

            // Skip if not all on same face
            if proj[0].face != proj[1].face
            || proj[0].face != proj[2].face
            || proj[0].face != proj[3].face {
                println!("lat={lat}, lon={lon} - SPANS MULTIPLE FACES: {},{},{},{}",
                    proj[0].face, proj[1].face, proj[2].face, proj[3].face);
                continue;
            }

            // Authalic spherical area
            let lat0_auth = Vgc::lat_geodetic_to_authalic(lat.to_radians(), &coef);
            let lat1_auth = Vgc::lat_geodetic_to_authalic((lat + 1.0).to_radians(), &coef);
            let dlon_rad  = 1.0_f64.to_radians();
            let spherical_area = r * r * dlon_rad
                * (lat1_auth.sin() - lat0_auth.sin()).abs();

            // Projected area via shoelace
            let (x1, y1) = (proj[0].coords.x, proj[0].coords.y);
            let (x2, y2) = (proj[1].coords.x, proj[1].coords.y);
            let (x3, y3) = (proj[2].coords.x, proj[2].coords.y);
            let (x4, y4) = (proj[3].coords.x, proj[3].coords.y);

            let projected_area = 0.5 * (
                (x1*y2 - x2*y1)
              + (x2*y3 - x3*y2)
              + (x3*y4 - x4*y3)
              + (x4*y1 - x1*y4)
            ).abs();

            let ratio = projected_area / spherical_area;
            println!("lat={lat:>3}, lon={lon:>5}, face={}, ratio={ratio:.4}",
                proj[0].face);
        }
    }

    // let coords = projection.geo_to_face(all.to_vec(), Some(&icosahedron));
    // println!("{:?}", coords);

    let distortion = projection.compute_distortion(38.68499, -9.49420, &icosahedron);
    println!("h: {} (expected: 0.7580403)", distortion.h);
    println!("k: {} (expected: 1.333174)", distortion.k);
    println!(
        "Angular deformation: {}° (expected: 33.045°)",
        distortion.angular_deformation
    );
    println!("Areal scale: {} (expected: ~1.0)", distortion.areal_scale);

    // let test_points = [
    //     (-9.494, 38.685), // Lisbon
    //     (0.0, 45.0),
    //     (30.0, 30.0),
    //     (0.0, 0.0),
    //     (0.0, 90.0), // North pole
    // ];

    // for (lon, lat) in test_points {
    //     let result =
    //         projection.geo_to_cartesian(vec![Point::new(lon, lat)], Some(&polyhedron), None);
    //     println!("Input:  lat={:.4}, lon={:.4}", lat, lon);
    //     println!(
    //         "Output: x={:.4}, y={:.4}, face={}",
    //         result[0].coords.x, result[0].coords.y, result[0].face
    //     );
    //     println!();
    // }

    // let p1 = projection.geo_to_cartesian(vec![Point::new(-9.494, 38.685)], Some(&polyhedron), None);
    // let p2 = projection.geo_to_cartesian(vec![Point::new(-9.500, 38.690)], Some(&polyhedron), None);
    // println!("p1: ({:.6}, {:.6})", p1[0].coords.x, p1[0].coords.y);
    // println!("p2: ({:.6}, {:.6})", p2[0].coords.x, p2[0].coords.y);
    // // These should be very close to each other
}
