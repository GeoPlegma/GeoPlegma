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
        polyhedron::{ArcLengths, Polyhedron, spherical_geometry::spherical_triangle_area},
        projections::traits::{DistortionMetrics, ForwardBary, ForwardCartesian, Projection},
    },
    utils::shape::{
        FACE_TEMPLATE_DOWN, FACE_TEMPLATE_UP, SUB_TRIANGLE_TEMPLATE, affine_transform_triangle,
        cartesian_2d_to_barycentric, compute_spherical_barycentric,
        get_subtriangle_vertices_in_face, map_subtriangle_vertices_to_face_2d, triangle,
        triangle3d_to_2d,
    },
};
use geo::{Coord, Point};

/// Implementation for Vertex Great Circle projection (or van Leeuwen Great Circle projection).
/// vgc - Vertex-oriented Great Circle projection.
/// Based on the slice and dice approach from this article:
/// http://dx.doi.org/10.1559/152304006779500687
pub struct Vgc;

impl Projection for Vgc {
    fn geo_to_face(
        &self,
        positions: Vec<Point>,
        polyhedron: Option<&Polyhedron>,
    ) -> Vec<ForwardBary> {
        let mut out: Vec<ForwardBary> = vec![];
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
                    let sub_triangle_3d = triangle(polyhedron, point_p, face).unwrap();

                    let face_vertices_3d = polyhedron.face_vertices(face).unwrap();
                    // calculating the arc lenghts from one of the vertices of the sub-triangle to point P
                    let ArcLengths {
                        ab, bp, ap, bc, ac, ..
                    } = polyhedron.arc_lengths(sub_triangle_3d.0, point_p);

                    // Parameterization values of the slice and dice projection.
                    let [xy, uv] = slice_and_dice(ac, ab, bc, ap, bp);

                    let face_edge_lengths = polyhedron.face_arc_lengths(face).unwrap();

                    let is_upward = if face % 2 == 0 { true } else { false };

                    // need to scale each face individually based on that specific face's spherical area
                    let face_spherical_area = spherical_triangle_area([
                        face_vertices_3d[0],
                        face_vertices_3d[1],
                        face_vertices_3d[2],
                    ])
                    .unwrap();

                    let face_2d_vertices = triangle3d_to_2d(
                        face_edge_lengths[0],
                        face_edge_lengths[1],
                        face_edge_lengths[2],
                        is_upward,
                        face_spherical_area,
                    );
                    // // Calculate barycentric coordinates within sub-triangle for P for the subtriangle ABC
                    // // Deduction using interpolation for the slice and dice, point D and then point P
                    // // Check the image here: https://raw.githubusercontent.com/GeoPlegma/GeoPlegma/refs/heads/master/gp-proj/src/assets/sub-triangles.png
                    // // P = B + (D - B) * xy => P = B * (1 - xy) + D * xy
                    // // So (1 - xy) and xy are the barycentric coordinates (the weights) of B and D respectively
                    // // Being that D = C + (A - C) * uv => D = A * uv + C * (1 - uv) then if you replace D in the previous equation
                    // // P = B * (1 - xy) + (A * uv + C * (1 - uv)) * xy => A × (xy × uv) + B × (1 - xy) + C × (xy × (1 - uv))
                    // // Therefore, the barycentric coordinates are:
                    // // - Weight for A (v_mid)**: `xy × uv`
                    // // - Weight for B (corner)**: `1 - xy`
                    // // - Weight for C (center)**: `xy × (1 - uv)`

                    // // let subtriangle_bary_u = xy * uv; // weight for v_mid (A)
                    // // let subtriangle_bary_v = 1.0 - xy; // weight for corner (B)
                    // // let subtriangle_bary_w = xy * (1.0 - uv); // weight for center (C)

                    // // Get barycentric coordinates of sub-triangle vertices with respect to face
                    // let sub_a_bary = compute_spherical_barycentric(
                    //     sub_triangle_3d.0[0], // v_mid
                    //     face_vertices_3d[0],
                    //     face_vertices_3d[1],
                    //     face_vertices_3d[2],
                    // );
                    // // Result: A = F0 × a0 + F1 × a1 + F2 × a2

                    // let sub_b_bary = compute_spherical_barycentric(
                    //     sub_triangle_3d.0[1], // corner
                    //     face_vertices_3d[0],
                    //     face_vertices_3d[1],
                    //     face_vertices_3d[2],
                    // );
                    // // Result: B = F0 × b0 + F1 × b1 + F2 × b2

                    // let sub_c_bary = compute_spherical_barycentric(
                    //     sub_triangle_3d.0[2], // center
                    //     face_vertices_3d[0],
                    //     face_vertices_3d[1],
                    //     face_vertices_3d[2],
                    // );
                    // // Result: C = F0 × c0 + F1 × c1 + F2 × c2

                    // // Compose: barycentric of P in face = weighted sum of sub-triangle vertices' barycentrics
                    // // Because barycentric coordinates are linear. When you have:
                    // // - P as a barycentric combination of A, B, C
                    // // - And A, B, C are themselves barycentric combinations of F0, F1, F2
                    // // You can "distribute" and get P directly as a barycentric combination of F0, F1, F2.
                    // // So then if we substitute the face-barycentric expressions for A, B, C back into the equation for P:
                    // // P = A × subtriangle_bary_u + B × subtriangle_bary_v + C × subtriangle_bary_w
                    // // Becomes (being that F(0,1,2) are the corners of the face):
                    // // P = (F0×a0 + F1×a1 + F2×a2) × subtriangle_bary_u + (F0×b0 + F1×b1 + F2×b2) × subtriangle_bary_v + (F0×c0 + F1×c1 + F2×c2) × subtriangle_bary_w
                    // // Rearranging by grouping F0, F1, F2:
                    // // P = F0 × (a0×subtriangle_bary_u  + b0×subtriangle_bary_v  + c0×subtriangle_bary_w ) + F1 × (a1×subtriangle_bary_u  + b1×subtriangle_bary_v  + c1×subtriangle_bary_w ) + F2 × (a2×subtriangle_bary_u  + b2×subtriangle_bary_v  + c2×subtriangle_bary_w )
                    // // let p_bary_u = subtriangle_a_bary_face.0 * subtriangle_bary_u
                    // //     + subtriangle_b_bary_face.0 * subtriangle_bary_v
                    // //     + subtriangle_c_bary_face.0 * subtriangle_bary_w;

                    // // let p_bary_v = subtriangle_a_bary_face.1 * subtriangle_bary_u
                    // //     + subtriangle_b_bary_face.1 * subtriangle_bary_v
                    // //     + subtriangle_c_bary_face.1 * subtriangle_bary_w;

                    // // let p_bary_w = subtriangle_a_bary_face.2 * subtriangle_bary_u
                    // //     + subtriangle_b_bary_face.2 * subtriangle_bary_v
                    // //     + subtriangle_c_bary_face.2 * subtriangle_bary_w;
                    // // Convert to 2D positions
                    // let sub_a_2d = (
                    //     face_2d_vertices[0].0 * sub_a_bary.0
                    //         + face_2d_vertices[1].0 * sub_a_bary.1
                    //         + face_2d_vertices[2].0 * sub_a_bary.2,
                    //     face_2d_vertices[0].1 * sub_a_bary.0
                    //         + face_2d_vertices[1].1 * sub_a_bary.1
                    //         + face_2d_vertices[2].1 * sub_a_bary.2,
                    // );
                    // let sub_b_2d = (
                    //     face_2d_vertices[0].0 * sub_b_bary.0
                    //         + face_2d_vertices[1].0 * sub_b_bary.1
                    //         + face_2d_vertices[2].0 * sub_b_bary.2,
                    //     face_2d_vertices[0].1 * sub_b_bary.0
                    //         + face_2d_vertices[1].1 * sub_b_bary.1
                    //         + face_2d_vertices[2].1 * sub_b_bary.2,
                    // );
                    // let sub_c_2d = (
                    //     face_2d_vertices[0].0 * sub_c_bary.0
                    //         + face_2d_vertices[1].0 * sub_c_bary.1
                    //         + face_2d_vertices[2].0 * sub_c_bary.2,
                    //     face_2d_vertices[0].1 * sub_c_bary.0
                    //         + face_2d_vertices[1].1 * sub_c_bary.1
                    //         + face_2d_vertices[2].1 * sub_c_bary.2,
                    // );

                    // let subtriangle_2d = [(1.0, 0.0), (0.0, 0.0), (1.0, bc * (PI / 6.0).sin())];
                    let sub_area = spherical_triangle_area([
                        sub_triangle_3d.0[0],
                        sub_triangle_3d.0[1],
                        sub_triangle_3d.0[2],
                    ])
                    .unwrap();

                    let subtriangle_2d = triangle3d_to_2d(ab, bc, ac, true, sub_area);
                    // Interpolate in 2D using slice-and-dice parameters
                    // D = C + (A - C) * uv
                    // let pd_x = sub_c_2d.0 + (sub_a_2d.0 - sub_c_2d.0) * uv;
                    // let pd_y = sub_c_2d.1 + (sub_a_2d.1 - sub_c_2d.1) * uv;
                    let pd_x =
                        subtriangle_2d[2].0 + (subtriangle_2d[1].0 - subtriangle_2d[2].0) * uv;
                    let pd_y =
                        subtriangle_2d[2].1 + (subtriangle_2d[1].1 - subtriangle_2d[2].1) * uv;

                    // P = B + (D - B) * xy
                    let p_x_local = subtriangle_2d[0].0 + (pd_x - subtriangle_2d[0].0) * xy;
                    let p_y_local = subtriangle_2d[0].1 + (pd_y - subtriangle_2d[0].1) * xy;

                    // STEP 4: Find where sub-triangle vertices A, B, C are in face 2D
                    // We need to know which sub-triangle this is to map vertices correctly
                    let (a_face_2d, b_face_2d, c_face_2d) = map_subtriangle_vertices_to_face_2d(
                        &sub_triangle_3d.0,
                        &face_vertices_3d,
                        &face_2d_vertices,
                    );

                    // STEP 5: Transform point from sub-triangle local to face coordinates
                    // Use affine transformation based on triangle correspondence
                    let p_face_2d = affine_transform_triangle(
                        (p_x_local, p_y_local),
                        subtriangle_2d,                    // source triangle [B, A, C]
                        [b_face_2d, a_face_2d, c_face_2d], // destination triangle
                    );

                    // println!("Point 2D: ({:.6}, {:.6})", p_x, p_y);
                    // Convert 2D Cartesian to barycentric w.r.t. face
                    let (bary_u, bary_v, bary_w) =
                        cartesian_2d_to_barycentric(p_face_2d, face_2d_vertices);

                    // Validate
                    const EPSILON: f64 = -1e-6;
                    if bary_u < EPSILON || bary_v < EPSILON || bary_w < EPSILON {
                        // println!(
                        //     "WARNING: Negative barycentric! Point ({}, {}), Face {}, Bary: ({}, {}, {})",
                        //     position.x(),
                        //     position.y(),
                        //     face,
                        //     bary_u,
                        //     bary_v,
                        //     bary_w
                        // );
                        continue;
                    }

                    let sum: f64 = bary_u + bary_v + bary_w;
                    if (sum - 1.0).abs() > 1e-6 {
                        println!("WARNING: Barycentric sum {} != 1.0", sum);
                        continue;
                    }

                    out.push(ForwardBary {
                        coords: Vector3D {
                            x: bary_u,
                            y: bary_v,
                            z: bary_w,
                        },
                        face: index,
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
        layout: Option<&dyn Layout>,
    ) -> Vec<ForwardCartesian> {
        let mut out: Vec<ForwardCartesian> = vec![];
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
                    let sub_triangle_3d = triangle(
                        polyhedron, point_p, // polyhedron.face_vertices(face).unwrap(),
                        face,
                    )
                    .unwrap();
                    let face_vertices_3d = polyhedron.face_vertices(face).unwrap();

                    // calculating the arc lenghts from one of the vertices of the sub-triangle to point P
                    let ArcLengths {
                        ab, bp, ap, bc, ac, ..
                    } = polyhedron.arc_lengths(sub_triangle_3d.0, point_p);

                    // Parameterization values of the slice and dice projection.
                    let [xy, uv] = slice_and_dice(ac, ab, bc, ap, bp);
                    /*
                    // // Get barycentric coordinates of sub-triangle vertices with respect to face
                    // let subtriangle_a_bary_face = compute_spherical_barycentric(
                    //     sub_triangle_3d.0[0], // v_mid
                    //     face_vertices_3d[0],
                    //     face_vertices_3d[1],
                    //     face_vertices_3d[2],
                    // );
                    // // Result: A = F0 × a0 + F1 × a1 + F2 × a2

                    // let subtriangle_b_bary_face = compute_spherical_barycentric(
                    //     sub_triangle_3d.0[1], // corner
                    //     face_vertices_3d[0],
                    //     face_vertices_3d[1],
                    //     face_vertices_3d[2],
                    // );
                    // // Result: B = F0 × b0 + F1 × b1 + F2 × b2

                    // let subtriangle_c_bary_face = compute_spherical_barycentric(
                    //     sub_triangle_3d.0[2], // center
                    //     face_vertices_3d[0],
                    //     face_vertices_3d[1],
                    //     face_vertices_3d[2],
                    // );

                    // let face_edge_lengths = polyhedron.face_arc_lengths(face).unwrap(); */

                    // let subtriangle_2d = triangle3d_to_2d(ab, bc, ac, true, sub_area);
                    /*   // // need to scale each face individually based on that specific face's spherical area
                          // let face_spherical_area = spherical_triangle_area([
                          //     face_vertices_3d[0],
                          //     face_vertices_3d[1],
                          //     face_vertices_3d[2],
                          // ])
                          // .unwrap();

                          // // println!("Face {}: edges={:?}, area={:.6}", face, face_edge_lengths, face_spherical_area);
                          // let face_2d_vertices = triangle3d_to_2d(
                          //     face_edge_lengths[0],
                          //     face_edge_lengths[1],
                          //     face_edge_lengths[2],
                          //     is_upward,
                          //     face_spherical_area,
                          // );
                    // println!("Face {} edge lengths: {:?}", face, face_2d_vertices);
                          // // println!("Face 2D vertices: {:?}", face_2d_vertices);
                          // let subtriangle_a_x = face_2d_vertices[0].0 * subtriangle_a_bary_face.0
                          //     + face_2d_vertices[1].0 * subtriangle_a_bary_face.1
                          //     + face_2d_vertices[2].0 * subtriangle_a_bary_face.2;

                          // let subtriangle_a_y = face_2d_vertices[0].1 * subtriangle_a_bary_face.0
                          //     + face_2d_vertices[1].1 * subtriangle_a_bary_face.1
                          //     + face_2d_vertices[2].1 * subtriangle_a_bary_face.2;

                          // let subtriangle_b_x = face_2d_vertices[0].0 * subtriangle_b_bary_face.0
                          //     + face_2d_vertices[1].0 * subtriangle_b_bary_face.1
                          //     + face_2d_vertices[2].0 * subtriangle_b_bary_face.2;

                          // let subtriangle_b_y = face_2d_vertices[0].1 * subtriangle_b_bary_face.0
                          //     + face_2d_vertices[1].1 * subtriangle_b_bary_face.1
                          //     + face_2d_vertices[2].1 * subtriangle_b_bary_face.2;

                          // let subtriangle_c_x = face_2d_vertices[0].0 * subtriangle_c_bary_face.0
                          //     + face_2d_vertices[1].0 * subtriangle_c_bary_face.1
                          //     + face_2d_vertices[2].0 * subtriangle_c_bary_face.2;

                          // let subtriangle_c_y = face_2d_vertices[0].1 * subtriangle_c_bary_face.0
                          //     + face_2d_vertices[1].1 * subtriangle_c_bary_face.1
                          //     + face_2d_vertices[2].1 * subtriangle_c_bary_face.2;*/

                    // ==== Interpolation ====
                    // Between A and C it gives point D
                    let pd_x = SUB_TRIANGLE_TEMPLATE[2].0
                        + (SUB_TRIANGLE_TEMPLATE[0].0 - SUB_TRIANGLE_TEMPLATE[2].0) * uv;
                    let pd_y = SUB_TRIANGLE_TEMPLATE[2].1
                        + (SUB_TRIANGLE_TEMPLATE[0].1 - SUB_TRIANGLE_TEMPLATE[2].1) * uv;
                    // Between D and B it gives point P
                    let p_x_local =
                        SUB_TRIANGLE_TEMPLATE[1].0 + (pd_x - SUB_TRIANGLE_TEMPLATE[1].0) * xy;
                    let p_y_local =
                        SUB_TRIANGLE_TEMPLATE[1].1 + (pd_y - SUB_TRIANGLE_TEMPLATE[1].1) * xy;
                    // ======================

                    let is_upward = if face % 2 == 0 { true } else { false };
                    let face_template = if is_upward {
                        FACE_TEMPLATE_UP
                    } else {
                        FACE_TEMPLATE_DOWN
                    };

                    // STEP 3: Get sub-triangle vertices in face coordinates
                    let sub_triangle_id = sub_triangle_3d.1;
                    let sub_vertices_in_face =
                        get_subtriangle_vertices_in_face(sub_triangle_id, face_template);

                    // STEP 4: Transform from sub-triangle local to face coordinates
                    let (p_x_face, p_y_face) = affine_transform_triangle(
                        (p_x_local, p_y_local),
                        SUB_TRIANGLE_TEMPLATE,
                        sub_vertices_in_face,
                    );

                    // STEP 5: Convert to meters
                    let r = 6371007.181;
                    out.push(ForwardCartesian {
                        coords: Coord {
                            x: p_x_face * r,
                            y: p_y_face * r,
                        },
                        face: index,
                    });

                    // in case the point is on the edge of two faces, we return the first face.
                    break;
                }
            }
        }
        out
    }

    fn cartesian_to_geo(&self, coords: Vec<Coord>) -> Point {
        todo!()
    }

    // @TODO - Needs to be reviewed
    // Calculate distortion and compare with Geocart values
    fn compute_distortion(&self, lat: f64, lon: f64, polyhedron: &Polyhedron) -> DistortionMetrics {
        let epsilon = 1e-5_f64; // degrees

        let center_xy =
            &self.geo_to_cartesian(vec![Point::new(lon, lat)], Some(polyhedron), None)[0];
        let north_xy =
            &self.geo_to_cartesian(vec![Point::new(lon, lat + epsilon)], Some(polyhedron), None)[0];
        let east_xy =
            &self.geo_to_cartesian(vec![Point::new(lon + epsilon, lat)], Some(polyhedron), None)[0];

        if center_xy.face != north_xy.face || center_xy.face != east_xy.face {
            return DistortionMetrics {
                h: f64::NAN,
                k: f64::NAN,
                angular_deformation: f64::NAN,
                areal_scale: f64::NAN,
            };
        }

        // epsilon in radians — coordinates are in meters, input was in degrees
        let eps_rad = epsilon.to_radians();

        let dx_dphi = (north_xy.coords.x - center_xy.coords.x) / eps_rad;
        let dy_dphi = (north_xy.coords.y - center_xy.coords.y) / eps_rad;
        let dx_dlambda = (east_xy.coords.x - center_xy.coords.x) / eps_rad;
        let dy_dlambda = (east_xy.coords.y - center_xy.coords.y) / eps_rad;

        // WGS84 radii of curvature (meters/radian)
        let a = 6378137.0_f64;
        let e2 = 0.00669437999014_f64;
        let lat_rad = lat.to_radians();
        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();

        let m = a * (1.0 - e2) / (1.0 - e2 * sin_lat.powi(2)).powf(1.5);
        let n = a / (1.0 - e2 * sin_lat.powi(2)).sqrt();

// Normalize derivatives by geodetic radii
let e  = dx_dlambda / (n * cos_lat);
let f  = dy_dlambda / (n * cos_lat);
let g  = dx_dphi    / m;
let h_ = dy_dphi    / m;

// Tissot: a and b are semi-axes of the indicatrix ellipse
let p = (e.powi(2) + f.powi(2)).sqrt();
let q = (g.powi(2) + h_.powi(2)).sqrt();
let t = e * g + f * h_;

let a_tissot = ((p + q).powi(2) - 2.0 * (e*h_ - f*g).abs() * (1.0 - (t / (p * q)).powi(2)).sqrt()).sqrt() / 2.0_f64.sqrt();
let b_tissot = ((p - q).powi(2) + 2.0 * (e*h_ - f*g).abs() * (1.0 - (t / (p * q)).powi(2)).sqrt()).sqrt() / 2.0_f64.sqrt();

let areal_scale = (e * h_ - f * g).abs();
let omega = 2.0 * ((a_tissot - b_tissot) / (a_tissot + b_tissot)).asin();

DistortionMetrics {
    h: a_tissot,
    k: b_tissot,
    angular_deformation: omega.to_degrees(),
    areal_scale,
}
    }
}

fn slice_and_dice(ac: f64, ab: f64, bc: f64, ap: f64, bp: f64) -> [f64; 2] {
    // Spherical angles for point B and point C
    let beta = ((ac.cos() - ab.cos() * bc.cos()) / (ab.sin() * bc.sin()))
        .clamp(-1.0, 1.0)
        .acos();
    let gamma = ((ab.cos() - bc.cos() * ac.cos()) / (bc.sin() * ac.sin()))
        .clamp(-1.0, 1.0)
        .acos();

    // ==== Slice and Dice formulas ====
    // angle ρ
    let rho: f64 =
        f64::acos(((ap.cos() - ab.cos() * bp.cos()) / (ab.sin() * bp.sin())).clamp(-1.0, 1.0));

    // 1. Calculate delta (δ)
    let delta = f64::acos(rho.sin() * ab.cos());

    // 2. Calculate the ratio of the spherical areas u and v
    let uv = ((beta + gamma - rho - delta) / (beta + gamma - PI / 2.0)).clamp(-1.0, 1.0);

    // 3. Calculate cos(x + y) by applying the spherical law of cosines
    // being that the x and y are the spherical lenghts from B to P and P to D, respectively.
    let cos_xp_y;
    if rho <= E.powi(-9) {
        // E = 2.71828...
        cos_xp_y = ab.cos();
    } else {
        cos_xp_y = 1.0 / (rho.tan() * delta.tan())
    }

    // 4. Calculate the ratio of the spherical areas x and y
    let xy = f64::sqrt((1.0 - bp.cos()) / (1.0 - cos_xp_y));

    [xy, uv]
}

// @TODO - new tests need to be added.
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
