// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{models::vector_3d::Vector3D, projections::layout::traits::Layout};
use geo::Coord;

use super::traits::{ArcLengths, Polyhedron};

pub const FACES: u8 = 20;

// pub const ORIENTATION_LAT: f64 =
// pub const ORIENTATION_LON: f64 =

#[derive(Default, Debug)]
pub struct Icosahedron {}

impl Polyhedron for Icosahedron {
    fn faces(&self) -> u8 {
        FACES
    }

    fn indices(&self) -> Vec<[u8; 3]> {
        todo!()
    }

    fn unit_vectors(&self) -> Vec<Vector3D> {
        todo!()
    }

    fn triangles(
        &self,
        _layout: &dyn Layout,
        _vector: Vector3D,
        _face_vectors: Vec<Vector3D>,
        _face_vertices: [(u8, u8); 3],
    ) -> ([Vector3D; 3], [Coord; 3]) {
        todo!()
    }

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
    fn triangle_arc_lengths(&self, triangle: [Vector3D; 3], vector: Vector3D) -> ArcLengths {
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

    fn is_point_in_triangle(&self, point: Vector3D, triangle: Vec<Vector3D>) -> bool {
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

    fn face_center(&self, vector1: Vector3D, vector2: Vector3D, vector3: Vector3D) -> Vector3D {
        Vector3D {
            x: (vector1.x + vector2.x + vector3.x) / 3.0,
            y: (vector1.y + vector2.y + vector3.y) / 3.0,
            z: (vector1.z + vector2.z + vector3.z) / 3.0,
        }
    }
}
