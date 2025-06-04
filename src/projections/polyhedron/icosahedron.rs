// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::{models::position::Position2D, projections::layout::traits::Layout};
use crate::models::vector_3d::Vector3D;

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
    ) -> ([Vector3D; 3], [Position2D; 3]) {
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

    fn is_point_in_triangle(&self, _point: Vector3D, _triangle: Vec<Vector3D>) -> bool {
        todo!()
    }

    /// Numerically stable angle between two unit vectors
    fn angle_between_unit(&self, _u: Vector3D, _v: Vector3D) -> f64 {
        todo!()
    }

    fn face_center(&self, vector1: Vector3D, vector2: Vector3D, vector3: Vector3D) -> Vector3D {
        Vector3D {
            x: (vector1.x + vector2.x + vector3.x) / 3.0,
            y: (vector1.y + vector2.y + vector3.y) / 3.0,
            z: (vector1.z + vector2.z + vector3.z) / 3.0,
        }
    }
}
