// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{
    models::vector_3d::Vector3D,
    projections::{layout::traits::Layout, polyhedron::traits::VertexIndices},
};
use geo::Coord;

use super::traits::{ArcLengths, Polyhedron};

pub const FACES: u8 = 20;

// pub const ORIENTATION_LAT: f64 =
// pub const ORIENTATION_LON: f64 =

#[derive(Default, Debug)]
pub struct Icosahedron {}

impl Polyhedron for Icosahedron {
    // The 12 points are symmetrically arranged on the sphere and lie at the same distance from the origin, forming a regular icosahedron
    // They are then nromalized in the sphere
    // **Returns the actual 3D positions of the three vertices for each face.**
    fn vertices(&self) -> Vec<Vector3D> {
        let phi = (1.0 + 5.0f64.sqrt()) / 2.0; // golden ratio
        vec![
            Vector3D {
                x: -1.0,
                y: phi,
                z: 0.0,
            }
            .normalize(),
            Vector3D {
                x: 1.0,
                y: phi,
                z: 0.0,
            }
            .normalize(),
            Vector3D {
                x: -1.0,
                y: -phi,
                z: 0.0,
            }
            .normalize(),
            Vector3D {
                x: 1.0,
                y: -phi,
                z: 0.0,
            }
            .normalize(),
            Vector3D {
                x: 0.0,
                y: -1.0,
                z: phi,
            }
            .normalize(),
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: phi,
            }
            .normalize(),
            Vector3D {
                x: 0.0,
                y: -1.0,
                z: -phi,
            }
            .normalize(),
            Vector3D {
                x: 0.0,
                y: 1.0,
                z: -phi,
            }
            .normalize(),
            Vector3D {
                x: phi,
                y: 0.0,
                z: -1.0,
            }
            .normalize(),
            Vector3D {
                x: phi,
                y: 0.0,
                z: 1.0,
            }
            .normalize(),
            Vector3D {
                x: -phi,
                y: 0.0,
                z: -1.0,
            }
            .normalize(),
            Vector3D {
                x: -phi,
                y: 0.0,
                z: 1.0,
            }
            .normalize(),
        ]
    }

    // **Returns the list of triangle faces as triplets of indices into the vertex array.**
    fn face_vertex_indices(&self) -> Vec<Vec<usize>> {
        vec![
            vec![0, 11, 5],
            vec![0, 5, 1],
            vec![0, 1, 7],
            vec![0, 7, 10],
            vec![0, 10, 11],
            vec![1, 5, 9],
            vec![5, 11, 4],
            vec![11, 10, 2],
            vec![10, 7, 6],
            vec![7, 1, 8],
            vec![3, 9, 4],
            vec![3, 4, 2],
            vec![3, 2, 6],
            vec![3, 6, 8],
            vec![3, 8, 9],
            vec![4, 9, 5],
            vec![2, 4, 11],
            vec![6, 2, 10],
            vec![8, 6, 7],
            vec![9, 8, 1],
        ]
    }

    /// Aproximate spherical centroid
    /// Fast, lies on the unit sphere, stable for icosahedral faces, hierachically consistent
    fn face_center(&self, face_id: usize) -> Vector3D {
        let indices = self.face_vertex_indices();
        let vertices = self.vertices();
        let face = &indices[face_id];
        let a = vertices[face[0]];
        let b = vertices[face[1]];
        let c = vertices[face[2]];

        let center = a + b + c;
        center.normalize()
    }
    
    /// Find the triangle face that contains the point on the sphere
    fn find_face(&self, point: Vector3D) -> Option<usize> {
        let vertices = self.vertices();
        for (face_idx, face) in self.face_vertex_indices().iter().enumerate() {
            let triangle: Vec<Vector3D> = face.iter().map(|&i| vertices[i]).collect();

            if self.is_point_in_face(point, triangle) {
                return Some(face_idx);
            }
        }
        None
    }

    // fn triangles(
    //     &self,
    //     _layout: &dyn Layout,
    //     _vector: Vector3D,
    //     _face_vectors: Vec<Vector3D>,
    //     _face_vertices: [(u8, u8); 3],
    // ) -> ([Vector3D; 3], [Coord; 3]) {
    //     todo!()
    // }

    /// Procedure to calculate arc lengths of the `triangle` with a point P (`vector` arc). To 90 degrees right triangle.
    /// 1. Compute center 3D vector of face
    /// 2. Compute center 2D point of face
    /// 3. Check which sub-triangle (out of 3) v falls into:
    ///     a. v2-v3
    ///     b. v3-v1
    ///     c. v1-v2
    /// 4. For that sub-triangle, compute midpoint (vMid, pMid)
    /// 5. Test which sub-sub-triangle v is in (with vCenter + vMid + corner)
    /// 6. Set the triangle vertex indices: [va, vb, vc] = [0, 1, 2]
    /// 7. Normalize vCenter, vMid
    fn face_arc_lengths(&self, triangle: [Vector3D; 3], vector: Vector3D) -> ArcLengths {
        // Vertex indices are [0, 1, 2]
        // Vertices for the 3D triangle that we want (v_mid: B, corner.0: A, v_center: C)
        // let v3d = [v_mid, corner.0, vector_center];
        // Vertices for the 2D triangle that we want
        // let p2d = [p_mid, corner.1, point_center];
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
            let v0 = ico.vertices()[face[0]];
            let v1 = ico.vertices()[face[1]];
            let v2 = ico.vertices()[face[2]];
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
