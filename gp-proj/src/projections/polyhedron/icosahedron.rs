// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Modified by Sunayana Ghosh (sunayanag@gmail.com)
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::f64::consts::PI;

use crate::{
    constants::PolyhedronConstants, models::vector_3d::Vector3D,
    projections::polyhedron::geometry::Face,
};

use super::polyhedron::Polyhedron;

/// Factory function to create an icosahedron with optimal orientation for DGGS
/// - Almost no vertices on land, which reduces distortion for land-based DGGS queries
/// by avoiding vertex-based singularities over populated areas.
/// - Two vertices on the poles, which ensures better symmetry for polar areas and
/// simplifies some projections.
/// - Rotated implementation optimized for equal-area projections.
/// That means this icosahedron is not a standard implementation but a rotated implementation to fit equal-area projections.
/// The other vertices are on northern and southern hemisphere in two equatorial rings, with alternating longitude.

pub fn new() -> Polyhedron {
    let vertices = create_vertices();
    let faces = create_faces();
    let num_edges = 30; // Icosahedron has 30 edges

    Polyhedron::new(vertices, faces, num_edges)
}

/// Create the 12 icosahedron vertices
fn create_vertices() -> Vec<Vector3D> {
    let mut vertices = Vec::with_capacity(12);
    let phi = PolyhedronConstants::golden_ratio();
    let z = 1.0 / (1.0 + phi.powi(2)).sqrt();
    let r = (1.0 - z.powi(2)).sqrt();

    // North Pole (Vertex 0)
    vertices.push(Vector3D {x: 0.0, y: 0.0, z: 1.0});

    // Upper ring (Vertices 1-5)
    for i in 0..5 {
        let angle = 2.0 * PI * (i as f64) / 5.0;
        vertices.push(Vector3D {
            x: r * angle.cos(),
            y: r * angle.sin(),
            z: z,
        });
    }

    // Lower ring (Vertices 6-10, rotated by 36°)
    for i in 0..5 {
        let angle  = 2.0 * PI * (i as f64) / 5.0 + PI / 5.0;
        vertices.push(Vector3D {
            x: r * angle.cos(),
            y: r * angle.sin(),
            z: -z,
        })
    }

    // South Pole (Vertex 11)
    vertices.push(Vector3D { x: 0.0, y: 0.0, z: -1.0});

    vertices
}

/// Create the 20 triangular faces of the icosahedron
fn create_faces() -> Vec<Face> {
    vec![
        Face::Triangle([0, 11, 5]), Face::Triangle([0, 5, 1]),
        Face::Triangle([0, 1, 7]), Face::Triangle([0, 7, 10]),
        Face::Triangle([0, 10, 11]), Face::Triangle([1, 5, 9]),
        Face::Triangle([5, 11, 4]),   Face::Triangle([11, 10, 2]),
        Face::Triangle([10, 7, 6]),   Face::Triangle([7, 1, 8]),
        Face::Triangle([3, 9, 4]),    Face::Triangle([3, 4, 2]),
        Face::Triangle([3, 2, 6]),    Face::Triangle([3, 6, 8]),
        Face::Triangle([3, 8, 9]),    Face::Triangle([4, 9, 5]),
        Face::Triangle([2, 4, 11]),   Face::Triangle([6, 2, 10]),
        Face::Triangle([8, 6, 7]),    Face::Triangle([9, 8, 1])
    ]
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icosahedron_creation() {
        let ico = new();
        assert_eq!(ico.num_vertices(), 12);
        assert_eq!(ico.num_faces(), 20);
        assert_eq!(ico.num_edges(), 30);
    }
    #[test]
    fn test_face_centers_on_unit_sphere() {
        let ico = new();

        for i in 0..ico.num_faces() {
            let center = ico.face_center(i);
            let norm = center.dot(center);
            assert!((norm - 1.0).abs() < 1e-5, "Face center {} not normalized", i);
        }
    }

    #[test]
    fn test_face_centers_inside_faces() {
        let ico = new();

        for i in 0..ico.num_faces() {
            let center = ico.face_center(i);
            assert!(ico.is_point_in_face(center, i), "Face center not inside face {}", i);
        }
    }
}
