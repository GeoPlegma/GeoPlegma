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
    projections::polyhedron::{
        Polyhedron,
        spherical_geometry::{self, barycentric_coordinates, spherical_triangle_area},
    },
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
pub fn triangle3d_to_2d(ab: f64, bc: f64, ac: f64, is_upward: bool) -> [(f64, f64); 3] {
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
}

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

// pub fn spherical_triangle_area(v0: Vector3D, v1: Vector3D, v2: Vector3D) -> f64 {
//     let a = spherical_geometry::stable_angle_between(v1, v2);
//     let b = spherical_geometry::stable_angle_between(v2, v0);
//     let c = spherical_geometry::stable_angle_between(v0, v1);

//     let s = (a + b + c) / 2.0;

//     let tan_e_over_4 =
//         ((s / 2.0).tan() * ((s - a) / 2.0).tan() * ((s - b) / 2.0).tan() * ((s - c) / 2.0).tan())
//             .sqrt();

//     4.0 * tan_e_over_4.atan()
// }

pub fn bary_to_cartesian(
    barycentric: Vector3D,
    origin_vertex: usize, // 0, 1, or 2
) -> (f64, f64) {
    const FACE_2D_TEMPLATE: [(f64, f64); 3] = [
        (0.0, 0.0),
        (1.1071487177940906, 0.0),
        (0.5535743588970453, 0.9585853315146595),
    ];

    let r_authalic = 1.0; // 6371007.181;

    // First compute with default origin (vertex 0)
    let u = barycentric.x;
    let v = barycentric.y;
    let w = 1.0 - u - v;

    let x = (FACE_2D_TEMPLATE[0].0 * u + FACE_2D_TEMPLATE[1].0 * v + FACE_2D_TEMPLATE[2].0 * w)
        * r_authalic;
    let y = (FACE_2D_TEMPLATE[0].1 * u + FACE_2D_TEMPLATE[1].1 * v + FACE_2D_TEMPLATE[2].1 * w)
        * r_authalic;

    // Translate to new origin
    let origin_offset = (
        FACE_2D_TEMPLATE[origin_vertex].0 * r_authalic * 0.0,
        FACE_2D_TEMPLATE[origin_vertex].1 * r_authalic * 0.0,
    );

    (x - origin_offset.0, y - origin_offset.1)
}

// face vertices [(1.0172219678978514, 0.0), (0.0, 0.0), (0.49405221144358774, 0.8891866757558156)]
//  subtriangle vertices [(0.49791064281623476, 0.5922059885716121), (1.0172219678978514, 0.0), (0.5017690741888817, 0.29522530138740866)]

// face vertices [(1.2566370614359172, 0.0), (0.0, 0.0), (0.7539822368615504, 1.0053096491487337)]
//  subtriangle vertices [(0.9722147547976886, 0.17194811203986515), (1.2566370614359172, 0.0), (0.6877924481594601, 0.3438962240797299)]

// face vertices [(1.0172219678978514, 0.0), (0.0, 0.0), (0.49405221144358746, 0.8891866757558157)]
//  subtriangle vertices [(0.7594955210433665, 0.14761265069370458), (0.49405221144358746, 0.8891866757558157), (0.5017690741888816, 0.2952253013874087)]

// [Forward { coords: COORD(0.8187121991892949 0.22255139041969554), face: 0, sub_triangle: 0 }, Forward { coords: COORD(1.1244248219039588 0.07992919136612857), face: 1, sub_triangle: 0 }, Forward { coords: COORD(0.6020272585050668 0.5857847703220402), face: 8, sub_triangle: 2 }]
