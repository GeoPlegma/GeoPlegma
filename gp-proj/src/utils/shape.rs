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
    projections::polyhedron::{Polyhedron, spherical_geometry},
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

// [Forward { coords: COORD(0.406882134198423 0.02037749900224545), face: 0, sub_triangle: 0 }, Forward { coords: COORD(0.1338295046010688 0.06516388444493568), face: 1, sub_triangle: 2 }, Forward { coords: COORD(0.4288013997674573 0.0049096528745127275), face: 8, sub_triangle: 2 }]
/// This will divide the icosahedron face in six equilateral triangles and get the triangle where the point is in
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

    let mid = Vector3D::mid(c, v0);

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

    // let (v_mid, corner, sub_triangle_id): (Vector3D, Vector3D, u8) =
    //     if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v2, v3]) {
    //         let v_mid = Vector3D::mid(v2, v3);
    //         if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v3])
    //         {
    //             (v_mid, v3, 1)
    //         } else {
    //             (v_mid, v2, 0)
    //         }
    //     } else if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v3, v1])
    //     {
    //         let v_mid = Vector3D::mid(v3, v1);
    //         if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v3])
    //         {
    //             (v_mid, v3, 3)
    //         } else {
    //             (v_mid, v1, 4)
    //         }
    //     } else {
    //         let v_mid = Vector3D::mid(v1, v2);
    //         if spherical_geometry::point_in_spherical_triangle(point_p, [vector_center, v_mid, v2])
    //         {
    //             (v_mid, v2, 6)
    //         } else {
    //             (v_mid, v1, 5)
    //         }
    //     };

    // ([v_mid, corner, vector_center], sub_triangle_id)
}
