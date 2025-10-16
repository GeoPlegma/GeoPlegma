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
        projections::traits::{Forward, Projection},
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

        // ABC
        let angle_beta: f64 = 36.0f64.to_radians();
        // BCA
        let angle_gamma: f64 = 60.0f64.to_radians();
        // BAC
        // let angle_alpha: f64 = PI / 2.0;

        for position in positions {
            let lon = position.x().to_radians();
            let lat = Self::lat_geodetic_to_authalic(
                position.y().to_radians(),
                &coef_fourier_geod_to_auth,
            );
            // Calculate 3d unit vectors for point P
            let point_p = Vector3D::from_array(Self::to_3d(lat, lon));

            // Triangle vertexes for local barycentric system (A,B,C)
            let (p0, p1, p2) = (
                Coord { x: 0.0_f64, y: 0.0 },
                Coord {
                    x: 0.5_f64,
                    y: (3.0_f64).sqrt() * 0.5,
                },
                Coord { x: 1.0_f64, y: 0.0 },
            );

            // starting from here, you need:
            // - the 3d point that you want to project
            // Polyhedron faces
            let faces_length = polyhedron.num_faces();
            for index in 0..faces_length {
                let face = usize::from(index);

                if polyhedron.is_point_in_face(point_p, index) {
                    // the icosahedron triangle gets divided into six equilateral triangles,
                    // and we find the one where the point is
                    let triangle_3d = triangles(
                        polyhedron,
                        point_p,
                        polyhedron.face_vertices(face).unwrap(),
                        face,
                    );

                    // need to find in which triangle the point is in
                    let ArcLengths { ab, bp, ap, .. } =
                        polyhedron.face_arc_lengths(triangle_3d, point_p);

                    // ==== Slice and Dice formulas ====
                    // angle ρ
                    let rho: f64 =
                        f64::acos(ap.cos() - ab.cos() * bp.cos()) / (ab.sin() * bp.sin());

                    // 1. Calculate delta (δ)
                    let delta = f64::acos(rho.sin() * ab.cos());

                    // 2. Calculate the ratio of the spherical areas u and v
                    let uv = (angle_beta + angle_gamma - rho - delta)
                        / (angle_beta + angle_gamma - PI / 2.0);

                    let cos_xp_y;
                    if rho <= E.powi(-9) {
                        cos_xp_y = ab.cos();
                    } else {
                        cos_xp_y = 1.0 / (rho.tan() * delta.tan())
                    }

                    let xy = f64::sqrt((1.0 - bp.cos()) / (1.0 - cos_xp_y));
                    // =================================

                    // ==== Interpolation ====

                    // Between A and C it gives point D
                    let pd_x = p2.x + (p0.x - p2.x) * uv;
                    let pd_y = p2.y + (p0.y - p2.y) * uv;

                    // Between D and B it gives point P
                    let p_x = pd_x + (pd_x - p1.x) * xy;
                    let p_y = pd_y + (pd_y - p1.y) * xy;
                    // ======================

                    out.push(Forward {
                        coords: Coord { x: p_x, y: p_y },
                        face: index,
                    });
                }
            }
        }

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
}

/// This will divide the icosahedron face in six equilateral triangles
fn triangles(
    polyhedron: &Polyhedron,
    point_p: Vector3D,
    face_vectors: Vec<Vector3D>,
    face_id: usize,
) -> [Vector3D; 3] {
    let (v1, v2, v3) = (face_vectors[0], face_vectors[1], face_vectors[2]);
    let mut vector_center = polyhedron.face_center(face_id);

    let (mut v_mid, corner): (Vector3D, Vector3D) =
        if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v2, v3]) {
            let v_mid = Vector3D::mid(v2, v3);
            if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v3])
            {
                (v_mid, v3)
            } else {
                (v_mid, v2)
            }
        } else if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v3, v1])
        {
            let v_mid = Vector3D::mid(v3, v1);
            if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v3])
            {
                (v_mid, v3)
            } else {
                (v_mid, v1)
            }
        } else {
            let v_mid = Vector3D::mid(v1, v2);
            if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v2])
            {
                (v_mid, v2)
            } else {
                (v_mid, v1)
            }
        };

    [v_mid, corner, vector_center]
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
    fn project_forward() {
        let position = Point::new(-9.222154, 38.695125);
        let projection = Vgc;
        let icosahedron = new();
        let result = projection.geo_to_bary(vec![position], Some(&icosahedron));
    }
}
