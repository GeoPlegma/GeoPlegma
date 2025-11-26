// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::f64::consts::{E, PI};

use crate::{
    constants::KarneyCoefficients,
    models::vector_3d::Vector3D,
    projections::{
        layout::traits::Layout,
        polyhedron::{ArcLengths, Polyhedron, spherical_geometry},
        projections::traits::{DistortionMetrics, Forward, Projection},
    },
};
use geo::{Coord, Point};

/// Implementation for Vertex Great Circle projection (or van Leeuwen Great Circle projection).
/// vgc - Vertex-oriented Great Circle projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgc;

impl Projection for Vgc {
    fn geo_to_bary(&self, positions: Vec<Point>, polyhedron: Option<&Polyhedron>) -> Vec<Forward> {
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
                    // the icosahedron triangle gets divided into six rectangle triangles,
                    // and we find the one where the point is
                    let triangle_3d = triangle(
                        polyhedron,
                        point_p,
                        polyhedron.face_vertices(face).unwrap(),
                        face,
                    );

                    // need to find in which triangle the point is in
                    let ArcLengths {
                        ab, bp, ap, bc, ac, ..
                    } = polyhedron.face_arc_lengths(triangle_3d.0, point_p);

                    // Map the 3D triangle to 2D
                    let triangle_2d = triangle3d_to_2d(ab, bc, ac);

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

                    // =================================
                    // ==== Interpolation ====
                    // Between A and C it gives point D
                    let pd_x = triangle_2d[2].0 + (triangle_2d[0].0 - triangle_2d[2].0) * uv;
                    let pd_y = triangle_2d[2].1 + (triangle_2d[0].1 - triangle_2d[2].1) * uv;

                    // Between D and B it gives point P
                    let p_x = triangle_2d[1].0 + (pd_x - triangle_2d[1].0) * xy;
                    let p_y = triangle_2d[1].1 + (pd_y - triangle_2d[1].1) * xy;
                    // ======================

                    out.push(Forward {
                        coords: Coord { x: p_x * 6378137.0, y: p_y *  6378137.0},
                        face: index,
                        sub_triangle: triangle_3d.1,
                    });

                    // in case the point is on the edge of two faces, we return the first face.
                    break;
                }
            }
        }
        println!("{:?}", out);

        out
    }
    fn bary_to_geo(&self, positions: Vec<Coord>) -> Point {
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
        let epsilon = 1e-7; // Small perturbation for numerical derivatives

        // Project the original point
        let center = &self.geo_to_bary(vec![Point::new(lon, lat)], Some(polyhedron))[0];

        // Perturb latitude (north-south)
        let north = &self.geo_to_bary(vec![Point::new(lon, lat + epsilon)], Some(polyhedron))[0];

        // Perturb longitude (east-west)
        let east = &self.geo_to_bary(vec![Point::new(lon + epsilon, lat)], Some(polyhedron))[0];

        // Calculate partial derivatives
        // dx/dφ, dy/dφ
        let dx_dphi = (north.coords.x - center.coords.x) / epsilon.to_radians();
        let dy_dphi = (north.coords.y - center.coords.y) / epsilon.to_radians();

        // dx/dλ, dy/dλ
        let dx_dlambda = (east.coords.x - center.coords.x) / epsilon.to_radians();
        let dy_dlambda = (east.coords.y - center.coords.y) / epsilon.to_radians();

        // Radius of curvature for WGS84
        let a = 6378137.0; // semi-major axis
        let e2 = 0.00669437999014; // first eccentricity squared
        let lat_rad = lat.to_radians();

        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();

        // Meridional radius of curvature
        let m = a * (1.0 - e2) / (1.0 - e2 * sin_lat * sin_lat).powf(1.5);

        // Prime vertical radius of curvature
        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();

        // Scale factors
        let h = (dx_dphi.powi(2) + dy_dphi.powi(2)).sqrt() / m;
        let k = (dx_dlambda.powi(2) + dy_dlambda.powi(2)).sqrt() / (n * cos_lat);

        // Angular deformation
        // sin(ω/2) = |h - k| / (h + k) where ω is the maximum angular distortion
        let sin_half_omega = (h - k).abs() / (h + k);
        let omega = 2.0 * sin_half_omega.asin();
        let angular_deformation_degrees = omega.to_degrees();

        DistortionMetrics {
            h,
            k,
            angular_deformation: angular_deformation_degrees,
            areal_scale: h * k, // For equal-area projections, this should be ~1.0
        }
    }
}

fn triangle3d_to_2d(ab: f64, bc: f64, ac: f64) -> [(f64, f64); 3] {
    // Place vertex B (triangle_3d[1] / corner) at origin
    let b_2d = (0.0, 0.0);

    // Place vertex A (triangle_3d[0] / v_mid) on the positive x-axis at distance ab
    let a_2d = (ab, 0.0);

    // Use law of cosines to find angle at B
    // cos(angle_B) = (ab² + bc² - ac²) / (2·ab·bc)
    let cos_angle_b = (bc.powi(2) + ac.powi(2) - ac.powi(2)) / (2.0 * bc * ac);
    let angle_b = cos_angle_b.clamp(-1.0, 1.0).acos();

    // Place vertex C (triangle_3d[2] / vector_center) using angle and distance bc
    let c_2d = (bc * angle_b.cos(), bc * angle_b.sin());

    [a_2d, b_2d, c_2d]
}

/// This will divide the icosahedron face in six equilateral triangles and get the triangle where the point is in
fn triangle(
    polyhedron: &Polyhedron,
    point_p: Vector3D,
    face_vectors: Vec<Vector3D>,
    face_id: usize,
) -> ([Vector3D; 3], u8) {
    let (v1, v2, v3) = (face_vectors[0], face_vectors[1], face_vectors[2]);
    let vector_center = polyhedron.face_center(face_id);

    let (v_mid, corner, sub_triangle_id): (Vector3D, Vector3D, u8) =
        if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v2, v3]) {
            let v_mid = Vector3D::mid(v2, v3);
            if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v3])
            {
                (v_mid, v3, 1)
            } else {
                (v_mid, v2, 0)
            }
        } else if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v3, v1])
        {
            let v_mid = Vector3D::mid(v3, v1);
            if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v3])
            {
                (v_mid, v3, 3)
            } else {
                (v_mid, v1, 4)
            }
        } else {
            let v_mid = Vector3D::mid(v1, v2);
            if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v2])
            {
                (v_mid, v2, 6)
            } else {
                (v_mid, v1, 5)
            }
        };

    ([v_mid, corner, vector_center], sub_triangle_id)
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
            projection.geo_to_bary(vec![p1, p2, p3, p4, p5, p6, p7, p8, p9], Some(&icosahedron));

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

        let results = projection.geo_to_bary(vec![lisbon, porto, madrid], Some(&icosahedron));

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

        let results = projection.geo_to_bary(points, Some(&icosahedron));

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

        let results = projection.geo_to_bary(points, Some(&icosahedron));

        // Should hit multiple different faces
        let unique_faces: std::collections::HashSet<_> = results.iter().map(|r| r.face).collect();

        println!("Unique faces at equator: {:?}", unique_faces);
        assert!(unique_faces.len() >= 5, "Should span multiple faces");
    }
}
