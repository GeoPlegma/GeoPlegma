// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{
    Vector3D,
    projections::polyhedron::{Polyhedron, spherical_geometry::{self, spherical_triangle_area}},
};

#[derive(Clone, Copy, Debug)]
struct Triangle {
    a: Vector3D,
    b: Vector3D,
    c: Vector3D,
}

impl Triangle {
    /// Create a new 3D vector
    pub fn new(a: Vector3D, b: Vector3D, c: Vector3D) -> Self {
        Self { a, b, c }
    }

    pub fn spherical_barycenter(&self) -> Vector3D {
        let sum = self.a + self.b + self.c;
        sum.normalize()
    }
}

/// This will divide the polyhedron face in equilateral triangles (each matches a vertice), then divides each triangle into two rectangular triangles, and gets the triangle where the point is in. Use for the van Leeuwen projection.
pub fn triangle(
    polyhedron: &Polyhedron,
    point_p: Vector3D,
    face_id: usize,
) -> Option<([Vector3D; 3], u8)> {
    let vertices = polyhedron.face_vertices(face_id)?;

    let n = vertices.len();
    if n < 3 {
        return None;
    }
    // Face center (spherical centroid)
    let center = polyhedron.face_center(face_id);

    // -------------------------------------------------
    // 1. Find the macro triangle (C, V[i], V[i+1])
    // -------------------------------------------------
    let mut found: Option<([Vector3D; 3], (usize, usize))> = None;

    for i in 0..n {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % n];

        let tri = [center, v1, v2];

        if spherical_geometry::point_in_spherical_triangle(point_p, tri) {
            found = Some((tri, (i, (i + 1) % n)));
            break;
        }
    }

    let (macro_tri, (i0, i1)) = found.or_else(|| {
        // fallback: find nearest macro triangle by minimum angle
        let mut best: Option<([Vector3D; 3], (usize, usize), f64)> = None;

        for i in 0..n {
            let v0 = vertices[i];
            let v1 = vertices[(i + 1) % n];
            let tri = [center, v0, v1];

            // Compute angle between center->point and center->mid-edge
            let mid = Vector3D::mid(v0, v1);
            let score = spherical_geometry::stable_angle_between(point_p, mid); // or dot-product distance

            match best {
                None => best = Some((tri, (i, (i + 1) % n), score)),
                Some((_, _, best_score)) if score < best_score => {
                    best = Some((tri, (i, (i + 1) % n), score))
                }
                _ => {}
            }
        }

        best.map(|(tri, id, _)| (tri, id))
    })?;

    // -------------------------------------------------
    // 2. Split macro triangle at midpoint of (C, V0)
    //    macro_tri = [C, V0, V1]
    // -------------------------------------------------
    let c = macro_tri[0];
    let v0 = macro_tri[1];
    let v1 = macro_tri[2];

    let mid = Vector3D::mid(v0, v1);

    // Left sub-triangle = (C, mid, V0)
    let left = [c, mid, v0];

    if spherical_geometry::point_in_spherical_triangle(point_p, left) {
        return Some(([mid, v0, c], i0 as u8));
    }

    // Right sub-triangle = (C, V1, mid)
    let right = [c, v1, mid];

    if spherical_geometry::point_in_spherical_triangle(point_p, right) {
        return Some(([mid, v1, c], i1 as u8));
    }

    // -------------------------------
    // 3. Fallback: choose closest subtriangle
    // -------------------------------
    let left = Triangle::new(c, v0, mid);
    let right = Triangle::new(c, v1, mid);
    let d_left = spherical_geometry::stable_angle_between(point_p, left.spherical_barycenter());
    let d_right = spherical_geometry::stable_angle_between(point_p, right.spherical_barycenter());

    if d_left < d_right {
        Some(([mid, v0, c], i0 as u8))
    } else {
        Some(([mid, v1, c], i1 as u8))
    }
}

// Map spherical triangle into a planar triangle.
pub fn triangle3d_to_2d(ab: f64, bc: f64, ac: f64, is_upward: bool, spherical_area: f64) -> [(f64, f64); 3] {
    let a01 = ab; // edge 0-1
    let a12 = bc; // edge 1-2
    let a20 = ac; // edge 2-0
    // Build triangle with v1 at origin
    // v1 at origin
    let v1 = (0.0, 0.0);

    // v0 on negative x-axis at distance a01
    let v0 = (-a01, 0.0);

    // v2 positioned using law of cosines
    // We know: a01 (v0 to v1), a12 (v1 to v2), a20 (v2 to v0)
    // Find angle at v1
    let cos_angle_v1 = (a01.powi(2) + a12.powi(2) - a20.powi(2)) / (2.0 * a01 * a12);
    let angle_v1 = cos_angle_v1.clamp(-1.0, 1.0).acos();

    let y_sign = if is_upward { 1.0 } else { -1.0 };

    // v2 at distance a12 from v1, at angle from negative x-axis
    let v2_x = -a12 * angle_v1.cos();
    let v2_y = y_sign * a12 * angle_v1.sin();
    let v2 = (v2_x, v2_y);

    [v1, v0, v2]
    //  let a01 = ab;
    // let a12 = bc;
    // let a20 = ac;
    
    // // Build triangle with edge lengths (temporarily)
    // let v1 = (0.0, 0.0);
    // let v0 = (-a01, 0.0);
    
    // let cos_angle_v1 = (a01.powi(2) + a12.powi(2) - a20.powi(2)) / (2.0 * a01 * a12);
    // let angle_v1 = cos_angle_v1.clamp(-1.0, 1.0).acos();
    
    // let y_sign = if is_upward { 1.0 } else { -1.0 };
    
    // let v2_x = -a12 * angle_v1.cos();
    // let v2_y = y_sign * a12 * angle_v1.sin();
    
    // // Calculate planar area of this triangle
    // let planar_area = 0.5 * (v0.0 * v2_y - v2_x * v0.1).abs();
    
    // // // Calculate spherical area using L'Huilier's theorem
    // // let s = (a01 + a12 + a20) / 2.0;
    // // let tan_e_over_4 = (
    // //     (s / 2.0).tan() *
    // //     ((s - a01) / 2.0).tan() *
    // //     ((s - a12) / 2.0).tan() *
    // //     ((s - a20) / 2.0).tan()
    // // ).sqrt();
    // // let spherical_area = 4.0 * tan_e_over_4.atan();
    
    // // Scale factor to make planar area equal spherical area
    // let scale = (spherical_area / planar_area).sqrt();
    
    // // Scale all vertices
    // let v0_scaled = (v0.0 * scale, v0.1 * scale);
    // let v1_scaled = (v1.0 * scale, v1.1 * scale);
    // let v2_scaled = (v2_x * scale, v2_y * scale);
    
    // [v1_scaled, v0_scaled, v2_scaled]
}

// @TODO - needs to be added to spherical geometry, the other function there is not behaving correctly
pub fn compute_spherical_barycentric(
    point: Vector3D,
    v0: Vector3D,
    v1: Vector3D,
    v2: Vector3D,
) -> (f64, f64, f64) {
    let total_area = spherical_triangle_area([v0, v1, v2]).unwrap();
    let area0 = spherical_triangle_area([point, v1, v2]).unwrap();
    let area1 = spherical_triangle_area([v0, point, v2]).unwrap();
    let area2 = spherical_triangle_area([v0, v1, point]).unwrap();

    (area0 / total_area, area1 / total_area, area2 / total_area)
}


/// Convert 2D Cartesian coordinates to barycentric coordinates
pub fn cartesian_2d_to_barycentric(
    point: (f64, f64),
    triangle: [(f64, f64); 3],
) -> (f64, f64, f64) {
    let [f0, f1, f2] = triangle;
    let (p_x, p_y) = point;

    // // Check if point is very close to any vertex (handle special case)
    // let vertex_tolerance = 1e-12;
    // if (p_x - v0.0).length() < vertex_tolerance {
    //     return (1.0, 0.0, 0.0);
    // }
    // if (point - v1).length() < vertex_tolerance {
    //     return Some((0.0, 1.0, 0.0));
    // }
    // if (point - v2).length() < vertex_tolerance {
    //     return Some((0.0, 0.0, 1.0));
    // }



    // Vectors from f0 to other vertices
    let v0 = (f1.0 - f0.0, f1.1 - f0.1);
    let v1 = (f2.0 - f0.0, f2.1 - f0.1);
    let v2 = (p_x - f0.0, p_y - f0.1);

    // Dot products
    let d00 = v0.0 * v0.0 + v0.1 * v0.1;
    let d01 = v0.0 * v1.0 + v0.1 * v1.1;
    let d11 = v1.0 * v1.0 + v1.1 * v1.1;
    let d20 = v2.0 * v0.0 + v2.1 * v0.1;
    let d21 = v2.0 * v1.0 + v2.1 * v1.1;

    let denom = d00 * d11 - d01 * d01;
    
    if denom.abs() < 1e-10 {
        // Degenerate triangle - return invalid
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    let bary_v = (d11 * d20 - d01 * d21) / denom;  // Weight for f1
    let bary_w = (d00 * d21 - d01 * d20) / denom;  // Weight for f2
    let bary_u = 1.0 - bary_v - bary_w;            // Weight for f0

    (bary_u, bary_v, bary_w)
}