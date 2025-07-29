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
        polyhedron::traits::{ArcLengths, Polyhedron},
        projections::traits::Projection,
    },
};
use geo::{Point, Coord};

/// Implementation for Vertex Great Circle projection (or van Leeuwen Great Circle projection).
/// vgc - Vertex-oriented Great Circle projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgc;

impl Projection for Vgc {
    fn forward(
        &self,
        positions: Vec<Point>,
        polyhedron: Option<&dyn Polyhedron>,
        layout: &dyn Layout,
    ) -> Vec<Coord> {
        let mut out: Vec<Coord> = vec![];
        let polyhedron = polyhedron.unwrap();

        // Need the coeficcients to convert from geodetic to authalic
        let coef_fourier_geod_to_auth = Self::fourier_coefficients(KarneyCoefficients::GEODETIC_TO_AUTHALIC);

        // get 3d vertices of the icosahedron (unit vectors)
        let ico_vectors = polyhedron.vertices();
        let triangles_ids = polyhedron.face_vertex_indices();

        // ABC
        let angle_beta: f64 = 36.0f64.to_radians();
        // BCA
        let angle_gamma: f64 = 60.0f64.to_radians();
        // BAC
        let angle_alpha: f64 = PI / 2.0;

        let v2d = layout.vertices();

        for position in positions {
            let lon = position.x().to_radians();
            let lat = Self::lat_geodetic_to_authalic(
                position.y().to_radians(),
                &coef_fourier_geod_to_auth,
            );
            // Calculate 3d unit vectors for point P
            let vector_3d = Vector3D::from_array(Self::to_3d(lat, lon));

            // starting from here, you need:
            // - the 3d point that you want to project
            // - the 3d vertexes of the icosahedron
            // - the 2d vertexes of the layout
            // Polyhedron faces
        //     let faces_length = polyhedron.faces();
        //     for index in 0..faces_length {
        //         let face = usize::from(index);
        //         let ids = triangles_ids[face];

        //         let triangle_3d = vec![
        //             ico_vectors[ids[0] as usize],
        //             ico_vectors[ids[1] as usize],
        //             ico_vectors[ids[2] as usize],
        //         ];
        //         if polyhedron.is_point_in_triangle(vector_3d, triangle_3d.clone()) {
        //             let (triangle_3d, triangle_2d) =
        //                 polyhedron.triangles(layout, vector_3d, triangle_3d, v2d[face]);
        //             let ArcLengths { ab, bp, ap, .. } =
        //                 polyhedron.triangle_arc_lengths(triangle_3d, vector_3d);

        //             // ==== Slice and Dice formulas ====
        //             // angle ρ
        //             let rho: f64 =
        //                 f64::acos(ap.cos() - ab.cos() * bp.cos()) / (ab.sin() * bp.sin());

        //             // 1. Calculate delta (δ)
        //             let delta = f64::acos(rho.sin() * ab.cos());

        //             // 2. Calculate u
        //             let uv = (angle_beta + angle_gamma - rho - delta)
        //                 / (angle_beta + angle_gamma - PI / 2.0);

        //             let cos_xp_y;
        //             if rho <= E.powi(-9) {
        //                 cos_xp_y = ab.cos();
        //             } else {
        //                 cos_xp_y = 1.0 / (rho.tan() * delta.tan())
        //             }

        //             let xy = f64::sqrt((1.0 - bp.cos()) / (1.0 - cos_xp_y));
        //             // =================================

        //             // ==== Interpolation ====
        //             // Triangle vertexes
        //             let (p0, p1, p2) = (&triangle_2d[0], &triangle_2d[1], &triangle_2d[2]);

        //             // Between A e o C it gives point D
        //             let pd_x = p2.x + (p0.x - p2.x) * uv;
        //             let pd_y = p2.y + (p0.y - p2.y) * uv;

        //             // Between D and B it gives point P
        //             let p_x = pd_x + (pd_x - p1.x) * xy;
        //             let p_y = pd_y + (pd_x - p1.y) * xy;
        //             // ======================

        //             out.push(Coord { x: p_x, y: p_y });
        //         }
        //     }
        }

        out
    }
    fn inverse(&self) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use geo::Point;


    #[test]
    fn test_point_creation() {
        let position = Point::new(-9.222154, 38.695125);
        assert_eq!(position.x(), -9.222154);
        assert_eq!(position.y(), 38.695125);
    }

    // Forward projection test disabled until Icosahedron implementation is complete
    // #[test]
    // fn project_forward() {
    //     let position = Point::new(-9.222154, 38.695125);
    //     let projection = Vgc;
    //     let result = projection.forward(vec![position], Some(&Icosahedron {}), &IcosahedronNet {});
    // }
}
