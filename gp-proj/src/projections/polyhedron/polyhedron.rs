// Copyright 2025 contributors to the GeoPlegmata project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::models::vector_3d::Vector3D;
use super::geometry::{Face, ArcLengths};
use super::spherical_geometry;

/// A concrete polyhedron with pre-computed geometric data.
/// This design separates data from operations for better performance
#[derive(Debug, Clone)]
pub struct Polyhedron {
    /// Pre-computed 3D vertices on the unit sphere
    vertices: Vec<Vector3D>,
    /// Face definitions using vertex indices
    faces: Vec<Face>,
    /// Pre-computed face centers for fast lookup
    face_centers: Vec<Vector3D>,
    /// Number of vertices (for constants access)
    num_vertices: usize,
    /// Number of edges (for constants access)
    num_edges: usize,
    /// Number of faces (for constants access)
    num_faces: usize,
}

impl Polyhedron {
     /// Create a new polyhedron with pre-computed data
    pub fn new(vertices: Vec<Vector3D>, faces: Vec<Face>, num_edges: usize) -> Self {
        let num_vertices = vertices.len();
        let num_faces = faces.len();

        // Pre-compute face centers
        let face_centers = faces.iter()
            .map(|face| {
                let face_vertices: Vec<Vector3D> = face.indices()
                    .iter()
                    .map(|&i| vertices[i])
                    .collect();

                // Compute spherical centroid by averaging and normalizing
                let sum: Vector3D = face_vertices.iter().fold(Vector3D::zero(), |acc, &v| acc + v);
                sum.normalize()
            })
            .collect();

        Self {
            vertices,
            faces,
            face_centers,
            num_vertices,
            num_edges,
            num_faces,
        }
    }

    // Data accessors (return references - no allocations)

    pub fn vertices(&self) -> &[Vector3D] {
        &self.vertices
    }

    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    pub fn face_centers(&self) -> &[Vector3D] {
        &self.face_centers
    }

    pub fn num_vertices(&self) -> usize {
        self.num_vertices
    }

    pub fn num_edges(&self) -> usize {
        self.num_edges
    }

    pub fn num_faces(&self) -> usize {
        self.num_faces
    }
    
    // Geometric operations (work on pre-computed data)

    /// Get the center of a specific face (O(1) lookup)
    pub fn face_center(&self, face_id: usize) -> Option<Vector3D> {
        self.face_centers.get(face_id).copied()
    }

    /// Find the face containing a point on the unit sphere
    pub fn find_face(&self, point: Vector3D) -> Option<usize> {
        for (face_idx, face) in self.faces.iter().enumerate() {
            let triangle: Vec<Vector3D> = face.indices()
                .iter()
                .map(|&i| self.vertices[i])
                .collect();

            if spherical_geometry::point_in_planar_triangle(point, [triangle[0], triangle[1], triangle[2]]) {
                return Some(face_idx);
            }
        }
        None
    }

    /// Get vertices of a specific face
    pub fn face_vertices(&self, face_id: usize) -> Option<Vec<Vector3D>> {
        self.faces.get(face_id).map(|face| {
            face.indices()
                .iter()
                .map(|&i| self.vertices[i])
                .collect()
        })
    }

    /// Compute arc lengths for a triangle and point
    pub fn face_arc_lengths(&self, triangle: [Vector3D; 3], point: Vector3D) -> ArcLengths {
        let [mid, corner, center] = triangle;
        ArcLengths {
            ab: spherical_geometry::stable_angle_between(corner, mid),
            bc: spherical_geometry::stable_angle_between(mid, center),
            ac: spherical_geometry::stable_angle_between(corner, center),
            ap: spherical_geometry::stable_angle_between(corner, point),
            bp: spherical_geometry::stable_angle_between(mid, point),
            cp: spherical_geometry::stable_angle_between(center, point),
        }
    }

      /// Check if point lies within a face
    pub fn is_point_in_face(&self, point: Vector3D, face_id: usize) -> bool {
        if let Some(face_vertices) = self.face_vertices(face_id) {
            spherical_geometry::point_in_planar_triangle(point, [face_vertices[0], face_vertices[1], face_vertices[2]])
        } else {
            false
        }
    }
}