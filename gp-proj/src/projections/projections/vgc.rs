// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::{
    f64::consts::{E, PI},
    os::linux::raw,
};

use crate::{
    constants::{KarneyCoefficients, PolyhedronConstants},
    models::vector_3d::Vector3D,
    projections::{
        layout::traits::Layout,
        polyhedron::{ArcLengths, Polyhedron, spherical_geometry::barycentric_coordinates},
        projections::traits::{DistortionMetrics, Forward, Projection},
    },
    utils::shape::{
        compute_spherical_barycentric, map_subtriangle_to_face_2d, triangle, triangle3d_to_2d,
    },
};
use geo::{Coord, Point};

/// Implementation for Vertex Great Circle projection (or van Leeuwen Great Circle projection).
/// vgc - Vertex-oriented Great Circle projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgc;
// Hardcoded face 2D template (same for all faces)
const FACE_2D_VERTICES: [(f64, f64); 3] = [
    (0.0, 0.0),
    (1.1071487177940906, 0.0),
    (0.5535743588970453, 0.9585853315146595),
];

// Hardcoded face 2D template (same for all faces)
const FACE_2D_VERTICES_DOWN: [(f64, f64); 3] = [
    (0.0, 0.0),
    (0.5535743588970453, 0.9585853315146595),
    (1.1071487177940906, 0.0),
];
impl Projection for Vgc {
    fn geo_to_face(&self, positions: Vec<Point>, polyhedron: Option<&Polyhedron>) -> Vec<Forward> {
        let mut out: Vec<Forward> = vec![];
        let polyhedron = polyhedron.unwrap();

        // Need the coeficcients to convert from geodetic to authalic
        let coef_fourier_geod_to_auth =
            Self::fourier_coefficients(KarneyCoefficients::GEODETIC_TO_AUTHALIC);

        for position in positions {
            let lon = position.x().to_radians();
            let lat = Self::lat_geodetic_to_authalic(
                position.y().to_radians(),
                &coef_fourier_geod_to_auth,
            );
            // Calculate 3d unit vectors for point P
            let point_p = Vector3D::from_array(Self::to_3d(lat, lon));

            // starting from here, you need:
            // - the 3d point that you want to project
            // Polyhedron faces
            let faces_length = polyhedron.num_faces();
            for index in 0..faces_length {
                let face = usize::from(index);

                if polyhedron.is_point_in_face(point_p, index) {
                    // Create face-level 2D coordinate system
                    let face_vertices_2d = polyhedron.face_to_2d_system(face);

                    // the icosahedron triangle gets divided into six rectangle triangles,
                    // and we find the one where the point is
                    let triangle_3d = triangle(
                        polyhedron, point_p, // polyhedron.face_vertices(face).unwrap(),
                        face,
                    )
                    .unwrap();
                    let face_vertices_3d = polyhedron.face_vertices(face).unwrap();
                    // calculating the arc lenghts from one of the vertices of the sub-triangle to point P
                    let ArcLengths {
                        ab, bp, ap, bc, ac, ..
                    } = polyhedron.arc_lengths(triangle_3d.0, point_p);

                    // Map the 3D sub-triangle to the face's 2D coordinate system
                    // Map the 3D sub-triangle to 2D
                    // let triangle_2d = triangle3d_to_2d(ab, bc, ac);

                    // Spherical angles for point B and point C
                    let beta = ((ac.cos() - ab.cos() * bc.cos()) / (ab.sin() * bc.sin()))
                        .clamp(-1.0, 1.0)
                        .acos();
                    let gamma = ((ab.cos() - bc.cos() * ac.cos()) / (bc.sin() * ac.sin()))
                        .clamp(-1.0, 1.0)
                        .acos();

                    // ==== Slice and Dice formulas ====
                    // angle ρ
                    let rho: f64 = f64::acos(
                        ((ap.cos() - ab.cos() * bp.cos()) / (ab.sin() * bp.sin())).clamp(-1.0, 1.0),
                    );

                    // 1. Calculate delta (δ)
                    let delta = f64::acos(rho.sin() * ab.cos());

                    // 2. Calculate the ratio of the spherical areas u and v
                    let uv = (beta + gamma - rho - delta) / (beta + gamma - PI / 2.0);

                    // 3. Calculate cos(x + y) by applying the spherical law of cosines
                    // being that the x and y are the spherical lenghts from B to P and P to D, respectively.
                    let cos_x_y = 1.0 / (rho.tan() * delta.tan());

                    // 4. Calculate the ratio of the spherical areas x and y
                    let xy = f64::sqrt((1.0 - bp.cos()) / (1.0 - cos_x_y));

                    // // ================================================================
                    // // JUST TESTING THE 2D LOCAL FACE SYSTEM
                    // let face_2d_vertices = if face % 2 == 0 {
                    //     FACE_2D_VERTICES_UP
                    // } else {
                    //     FACE_2D_VERTICES_DOWN
                    // };

                    // // // Map the sub-triangle to the face's 2D coordinate system
                    // let triangle_2d = map_subtriangle_to_face_2d(
                    //     triangle_3d.0,
                    //     face_vertices_3d.clone(),
                    //     face_2d_vertices,
                    // );

                    // // =================================
                    // // ==== Interpolation ====
                    // // Between A and C it gives point D
                    // let pd_x = triangle_2d[2].0 + (triangle_2d[0].0 - triangle_2d[2].0) * uv;
                    // let pd_y = triangle_2d[2].1 + (triangle_2d[0].1 - triangle_2d[2].1) * uv;

                    // // Between D and B it gives point P
                    // let p_x = triangle_2d[1].0 + (pd_x - triangle_2d[1].0) * xy;
                    // let p_y = triangle_2d[1].1 + (pd_y - triangle_2d[1].1) * xy;
                    // // ======================
                    // // ================================================================

                    let face_vertices = [
                        face_vertices_3d[0].clone(),
                        face_vertices_3d[1].clone(),
                        face_vertices_3d[2].clone(),
                    ];
                    //     face_vertices_2d,
                    // );

                    // Calculate barycentric coordinates within sub-triangle
                    let sub_bary_u = xy * uv; // weight for v_mid (A)
                    let sub_bary_v = 1.0 - xy; // weight for corner (B)
                    let sub_bary_w = xy * (1.0 - uv); // weight for center (C)

                    // Get barycentric coordinates of sub-triangle vertices with respect to face
                    let sub_vertex_0_bary = compute_spherical_barycentric(
                        triangle_3d.0[0], // v_mid
                        face_vertices_3d[0].clone(),
                        face_vertices_3d[1].clone(),
                        face_vertices_3d[2].clone(),
                    );

                    let sub_vertex_1_bary = compute_spherical_barycentric(
                        triangle_3d.0[1], // corner
                        face_vertices_3d[0].clone(),
                        face_vertices_3d[1].clone(),
                        face_vertices_3d[2].clone(),
                    );

                    let sub_vertex_2_bary = compute_spherical_barycentric(
                        triangle_3d.0[2], // center
                        face_vertices_3d[0].clone(),
                        face_vertices_3d[1].clone(),
                        face_vertices_3d[2].clone(),
                    );
                    // println!(
                    //     "face vertices {:?} \n subtriangle vertices {:?} \n",
                    //     sub_bary_w, sub_vertex_0_bary
                    // );

                    // transform to 2D face coordinate system

                    // Compose: barycentric of P in face = weighted sum of sub-triangle vertices' barycentrics
                    let face_bary_u = sub_vertex_0_bary.0 * sub_bary_u
                        + sub_vertex_1_bary.0 * sub_bary_v
                        + sub_vertex_2_bary.0 * sub_bary_w;

                    let face_bary_v = sub_vertex_0_bary.1 * sub_bary_u
                        + sub_vertex_1_bary.1 * sub_bary_v
                        + sub_vertex_2_bary.1 * sub_bary_w;

                    let face_bary_w = sub_vertex_0_bary.2 * sub_bary_u
                        + sub_vertex_1_bary.2 * sub_bary_v
                        + sub_vertex_2_bary.2 * sub_bary_w;

                    println!(
                        "barycentric coordinates {:?} \n 
                        weighted sum {}",
                        // projected point in 2D {:?} \n
                        Vector3D {
                            x: face_bary_u,
                            y: face_bary_v,
                            z: face_bary_w,
                        },
                        // [p_x, p_y],
                        face_bary_u + face_bary_v + face_bary_w
                    );

                    out.push(Forward {
                        coords: Vector3D {
                            x: face_bary_u,
                            y: face_bary_v,
                            z: face_bary_w,
                        },
                        face: index,
                        sub_triangle: triangle_3d.1,
                    });

                    // in case the point is on the edge of two faces, we return the first face.
                    break;
                }
            }
        }

        out
    }
    fn face_to_geo(&self, positions: Vec<Coord>) -> Point {
        todo!()
    }

    fn geo_to_cartesian(
        &self,
        positions: Vec<Point>,
        polyhedron: Option<&Polyhedron>,
        layout: &dyn Layout,
    ) -> Vec<Forward> {
        todo!()
    }

    fn cartesian_to_geo(&self, coords: Vec<Coord>) -> Point {
        todo!()
    }

    // Calculate distortion and compare with Geocart values
    fn compute_distortion(&self, lat: f64, lon: f64, polyhedron: &Polyhedron) -> DistortionMetrics {
let face_2d_vertices: [(f64, f64); 3] = [
    (0.0, 0.0),
    (2.0 * (1.0 / PolyhedronConstants::golden_ratio()).asin(), 0.0),
    (0.5535743588970453, 0.9585853315146595),
];

        let r_authalic = 6371007.181;
        let epsilon = 1e-7;



         let coef_fourier_geod_to_auth =
            Self::fourier_coefficients(KarneyCoefficients::AUTHALIC_TO_GEODETIC);

            let lat_geodetic = Self::lat_authalic_to_geodetic(
                lat.to_radians(),
                &coef_fourier_geod_to_auth,
            );// Project the original point
        let center_bary = &self.geo_to_face(vec![Point::new(lon, lat)], Some(polyhedron))[0];

        // Perturb latitude (north-south)
        let north_bary =
            &self.geo_to_face(vec![Point::new(lon, lat + epsilon)], Some(polyhedron))[0];

        // Perturb longitude (east-west)
        let east_bary =
            &self.geo_to_face(vec![Point::new(lon + epsilon, lat)], Some(polyhedron))[0];

        // Handle face discontinuities
        if center_bary.face != north_bary.face || center_bary.face != east_bary.face {
            // Point is near face boundary, derivatives unreliable
            return DistortionMetrics {
                h: f64::NAN,
                k: f64::NAN,
                angular_deformation: f64::NAN,
                areal_scale: f64::NAN,
            };
        }

        // Convert barycentric to Cartesian in radians
        let to_cartesian = |bary: &Forward| -> (f64, f64) {
            let u = bary.coords.x;
            let v = bary.coords.y;
            let w = 1.0 - u - v;

            let x =
                face_2d_vertices[0].0 * u + face_2d_vertices[1].0 * v + face_2d_vertices[2].0 * w;
            let y =
                face_2d_vertices[0].1 * u + face_2d_vertices[1].1 * v + face_2d_vertices[2].1 * w;

            (x, y)
        };

        let center_xy = to_cartesian(&center_bary);
        let north_xy = to_cartesian(&north_bary);
        let east_xy = to_cartesian(&east_bary);

        // Derivatives in radians per radian
        let dx_dphi_rad = (north_xy.0 - center_xy.0) / epsilon.to_radians();
        let dy_dphi_rad = (north_xy.1 - center_xy.1) / epsilon.to_radians();

        let dx_dlambda_rad = (east_xy.0 - center_xy.0) / epsilon.to_radians();
        let dy_dlambda_rad = (east_xy.1 - center_xy.1) / epsilon.to_radians();

        // Convert to meters
        let dx_dphi = dx_dphi_rad * r_authalic;
        let dy_dphi = dy_dphi_rad * r_authalic;
        let dx_dlambda = dx_dlambda_rad * r_authalic;
        let dy_dlambda = dy_dlambda_rad * r_authalic;

        // WGS84 ellipsoid parameters for GEODETIC coordinates
        let a = 6378137.0;
        let e2 = 0.00669437999014;
        let lat_rad = lat_geodetic.to_radians();

        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();

        // Radii of curvature on the ellipsoid
        let m = a * (1.0 - e2) / (1.0 - e2 * sin_lat.powi(2)).powf(1.5);
        let n = a / (1.0 - e2 * sin_lat.powi(2)).sqrt();

        // Scale factors
        let h = (dx_dphi.powi(2) + dy_dphi.powi(2)).sqrt() / m;
        let k = (dx_dlambda.powi(2) + dy_dlambda.powi(2)).sqrt() / (n * cos_lat);

        // Angular deformation
        let a_tissot = ((h.powi(2) + k.powi(2)) / 2.0).sqrt();
        let b_tissot = (h * k).sqrt();

        let sin_half_omega = (a_tissot - b_tissot) / (a_tissot + b_tissot);
        let omega = 2.0 * sin_half_omega.asin();

        DistortionMetrics {
            h,
            k,
            angular_deformation: omega.to_degrees(),
            areal_scale: h * k,
        }
    }
}

#[cfg(test)]
mod tests {
    use geo::Point;

    use crate::projections::{
        polyhedron::icosahedron::{self, new},
        projections::{traits::Projection, vgc::Vgc},
    };

    #[test]
    fn test_point_creation() {
        let position = Point::new(-9.222154, 38.695125);
        assert_eq!(position.x(), -9.222154);
        assert_eq!(position.y(), 38.695125);
    }

    // Forward projection test disabled until Icosahedron implementation is complete
    #[test]
    fn test_project_forward() {
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
        let result =
            projection.geo_to_face(vec![p1, p2, p3, p4, p5, p6, p7, p8, p9], Some(&icosahedron));

        assert_eq!(result[0].face, 8);
        assert_eq!(result[1].face, 6);
        assert_eq!(result[2].face, 3);
        assert_eq!(result[3].face, 16);
        assert_eq!(result[4].face, 15);
        assert_eq!(result[5].face, 16);
        assert_eq!(result[6].face, 12);
        assert_eq!(result[7].face, 11);
        assert_eq!(result[8].face, 0);
    }

    #[test]
    fn test_spatial_consistency() {
        let projection = Vgc;
        let icosahedron = new();
        // Test points
        let lisbon = Point::new(-9.49420, 38.68499);
        let porto = Point::new(-8.61099, 41.14961); // ~300km north of Lisbon
        let madrid = Point::new(-3.70379, 40.41678); // ~500km east of Lisbon

        let results = projection.geo_to_face(vec![lisbon, porto, madrid], Some(&icosahedron));

        // Check they're on reasonable faces
        println!("Lisbon face: {}", results[0].face);
        println!("Porto face: {}", results[1].face);
        println!("Madrid face: {}", results[2].face);

        // Porto should be on same or adjacent face to Lisbon
        // (they're only 300km apart)
        assert!(
            results[0].face == results[1].face
                || icosahedron.are_faces_adjacent(results[0].face, results[1].face)
        );
    }

    #[test]
    fn test_pole_behavior() {
        let projection = Vgc;
        let icosahedron = new();

        // Points around the pole should be on adjacent faces
        let points = vec![
            Point::new(0.0, 89.0),
            Point::new(72.0, 89.0),
            Point::new(144.0, 89.0),
            Point::new(216.0, 89.0),
            Point::new(288.0, 89.0),
        ];

        let results = projection.geo_to_face(points, Some(&icosahedron));

        // All should be near pole (check they're on the 5 faces around the north pole)
        for (i, result) in results.iter().enumerate() {
            println!(
                "Point {} - Face: {}, Coords: {:?}",
                i, result.face, result.coords
            );
            let is_in_north_pole = match result.face {
                0 | 2 | 4 | 6 | 8 => true,
                _ => false,
            };
            assert!(is_in_north_pole, "Its not on the north pole");
        }
    }

    #[test]
    fn test_equator_distribution() {
        let projection = Vgc;
        let icosahedron = new();

        // Points evenly distributed around equator
        let points: Vec<Point> = (0..10).map(|i| Point::new(i as f64 * 36.0, 0.0)).collect();

        let results = projection.geo_to_face(points, Some(&icosahedron));

        // Should hit multiple different faces
        let unique_faces: std::collections::HashSet<_> = results.iter().map(|r| r.face).collect();

        println!("Unique faces at equator: {:?}", unique_faces);
        assert!(unique_faces.len() >= 5, "Should span multiple faces");
    }
    #[test]
    fn test_distortion() {
        let projection = Vgc;
        let icosahedron = new();
        let distortion = projection.compute_distortion(38.68499, -9.49420, &icosahedron);
        println!("h: {} (expected: 0.7580403)", distortion.h);
        println!("k: {} (expected: 1.333174)", distortion.k);
        println!(
            "Angular deformation: {}° (expected: 33.045°)",
            distortion.angular_deformation
        );
        println!(
            "Areal scale: {} (expected: ~1.0 for equal-area)",
            distortion.areal_scale
        );
    }
}
