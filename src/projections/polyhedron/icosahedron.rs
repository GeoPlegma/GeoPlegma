// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::f64::consts::PI;

use crate::{
    constants::PolyhedronConstants, models::vector_3d::Vector3D,
    projections::polyhedron::traits::Face,
};
use geo::Coord;

use super::traits::{ArcLengths, Polyhedron};

#[derive(Default, Debug)]
pub struct Icosahedron {}

/// This icosahedron implementation tries to have:
/// - Almost no vertices on land, which reduces distortion for land-based DGGS queries
/// by avoiding vertex-based singularities over populated areas.
/// - Two vertices on the poles, which ensures better symmetry for polar areas and
/// simplifies some projections.
/// That means this icosahedron is not a standard implementation but a rotated implementation to fit equal-area projections.
/// The other vertices are on northern and southern hemisphere in two equatorial rings, with alternating longitude.
impl Polyhedron for Icosahedron {
    /// The 12 points are symmetrically arranged on the sphere and lie at the same distance from the origin, forming a regular icosahedron
    /// They are then normalized in the sphere
    /// **Returns the actual 3D positions of the three vertices for each face.**
    fn vertices(&self) -> Vec<Vector3D> {
        let mut vertices = Vec::with_capacity(12);
        let phi = PolyhedronConstants::golden_ratio(); // golden ratio
        let z = 1.0 / (1.0 + phi.powi(2)).sqrt(); // Height (z) from center to top/bottom for the other 10 points
        let r = (1.0 - z.powi(2)).sqrt(); // Radius of the ring

        // === Vertex 0: North Pole ===
        vertices.push(Vector3D {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        });

        // === Vertices 1–5: Upper ring ===
        for i in 0..5 {
            let angle = 2.0 * PI * (i as f64) / 5.0;
            vertices.push(Vector3D {
                x: r * angle.cos(),
                y: r * angle.sin(),
                z: z,
            });
        }

        // === Vertices 6–10: Lower ring (rotated by 36°) ===
        for i in 0..5 {
            let angle = 2.0 * PI * (i as f64) / 5.0 + PI / 5.0; // 36° offset
            vertices.push(Vector3D {
                x: r * angle.cos(),
                y: r * angle.sin(),
                z: -z,
            });
        }

        // === Vertex 11: South Pole ===
        vertices.push(Vector3D {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        });

        vertices
    }

    /// **Returns the list of triangle faces as triplets of indices into the vertex array.**
    fn face_vertex_indices(&self) -> Vec<Face> {
        // want to avoid a loop here for performance reasons
        vec![
            Face::Triangle([0, 11, 5]),
            Face::Triangle([0, 5, 1]),
            Face::Triangle([0, 1, 7]),
            Face::Triangle([0, 7, 10]),
            Face::Triangle([0, 10, 11]),
            Face::Triangle([1, 5, 9]),
            Face::Triangle([5, 11, 4]),
            Face::Triangle([11, 10, 2]),
            Face::Triangle([10, 7, 6]),
            Face::Triangle([7, 1, 8]),
            Face::Triangle([3, 9, 4]),
            Face::Triangle([3, 4, 2]),
            Face::Triangle([3, 2, 6]),
            Face::Triangle([3, 6, 8]),
            Face::Triangle([3, 8, 9]),
            Face::Triangle([4, 9, 5]),
            Face::Triangle([2, 4, 11]),
            Face::Triangle([6, 2, 10]),
            Face::Triangle([8, 6, 7]),
            Face::Triangle([9, 8, 1]),
        ]
    }

    /// Aproximate spherical centroid
    /// Fast, lies on the unit sphere, stable for icosahedral faces, hierachically consistent
    /// **Gets the center point of the face**
    fn face_center(&self, face_id: usize) -> Vector3D {
        let faces = self.face_vertex_indices();
        let vertices = self.vertices();
        let face = faces[face_id].indices();
        let a = vertices[face[0]];
        let b = vertices[face[1]];
        let c = vertices[face[2]];

        let center = a + b + c;
        center.normalize()
    }

    /// We are looping through the faces till we find the point.
    /// **Finds the triangle face that contains the point on the sphere**
    fn find_face(&self, point: Vector3D) -> Option<usize> {
        let vertices = self.vertices();
        for (face_idx, face) in self.face_vertex_indices().iter().enumerate() {
            let triangle: Vec<Vector3D> = face.indices().iter().map(|&i| vertices[i]).collect();

            if self.is_point_in_face(point, triangle) {
                return Some(face_idx);
            }
        }
        None
    }

    /// TODO - needs to be reviewed for later PR
    /// Procedure to calculate arc lengths of the `triangle` with a point P (`vector` arc). To 90 degrees right triangle.
    fn face_arc_lengths(&self, triangle: [Vector3D; 3], vector: Vector3D) -> ArcLengths {
        let [mid, corner, center] = triangle;
        ArcLengths {
            ab: self.angle_between_unit(corner, mid),
            bc: self.angle_between_unit(mid, center),
            ac: self.angle_between_unit(corner, center),
            ap: self.angle_between_unit(corner, vector),
            bp: self.angle_between_unit(mid, vector),
            cp: self.angle_between_unit(center, vector),
        }
    }

    /// Uses barycentric coordinates
    /// **Find if point is in a face**
    fn is_point_in_face(&self, point: Vector3D, triangle: Vec<Vector3D>) -> bool {
        if triangle.len() != 3 {
            return false;
        }

        // For spherical triangles on icosahedron, use barycentric coordinates
        // adapted for the unit sphere
        let v0 = triangle[0];
        let v1 = triangle[1];
        let v2 = triangle[2];

        // Convert to barycentric coordinates
        let v0v1 = v1 - v0;
        let v0v2 = v2 - v0;
        let v0p = point - v0;

        let dot00 = v0v2.dot(v0v2);
        let dot01 = v0v2.dot(v0v1);
        let dot02 = v0v2.dot(v0p);
        let dot11 = v0v1.dot(v0v1);
        let dot12 = v0v1.dot(v0p);

        // Compute barycentric coordinates
        let denom = dot00 * dot11 - dot01 * dot01;
        if denom.abs() < 1e-10 {
            return false; // Degenerate triangle
        }

        let inv_denom = 1.0 / denom;
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        // Point is in triangle if all barycentric coordinates are non-negative
        u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
    }

    /// Numerically stable angle between two unit vectors
    /// Uses atan2 method for better numerical stability than acos
    /// **Return the angle between unit vectors**
    fn angle_between_unit(&self, u: Vector3D, v: Vector3D) -> f64 {
        // For unit vectors, use the cross product magnitude and dot product
        // with atan2 for numerical stability
        let cross = u.cross(v);
        let cross_magnitude = cross.length();
        let dot = u.dot(v);

        // atan2 handles all quadrants correctly and is more stable than acos
        cross_magnitude.atan2(dot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_center() {
        let ico = Icosahedron {};
        let faces = ico.face_vertex_indices();

        for (i, face) in faces.iter().enumerate() {
            let v0 = ico.vertices()[face.indices()[0]];
            let v1 = ico.vertices()[face.indices()[1]];
            let v2 = ico.vertices()[face.indices()[2]];
            let center = ico.face_center(i);

            // Check if center it's on the unit sphere
            let dot = center.dot(center);
            assert!(
                (dot - 1.0).abs() < 1e-5,
                "Face center {} not normalized: norm = {:?}",
                i,
                center
            );

            // Check if center lies inside the triangle
            assert!(
                ico.is_point_in_face(center, [v0, v1, v2].to_vec()),
                "Face center not inside triangle face {}",
                i
            );
        }
    }
}
