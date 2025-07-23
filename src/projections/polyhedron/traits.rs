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

pub enum VertexIndices {
    Triangles(Vec<[usize; 3]>),
    Cubes(Vec<[usize; 4]>),
    Pentagons(Vec<[usize; 5]>),
}

pub trait Polyhedron {
    /// Return the actual 3D vertices of each face.
    fn vertices(&self) -> Vec<Vector3D>;
    /// Return index triplets of the icosahedron faces.
    fn face_vertex_indices(&self) -> Vec<Vec<usize>>;
    /// Compute the centroid of a triangle face.
    fn face_center(&self, face_id: usize) -> Vector3D;
    /// Given a point on the unit sphere, return the face index that contains it.
    fn find_face(&self, point: Vector3D) -> Option<usize>;
    // fn unit_vectors(&self) -> Vec<Vector3D>;
    // fn triangles(
    //     &self,
    //     layout: &dyn Layout,
    //     vector: Vector3D,
    //     face_vectors: Vec<Vector3D>,
    //     face_vertices: [(u8, u8); 3],
    // ) -> ([Vector3D; 3], [Coord; 3]);
    /// Compute spherical arc lengths between point P and the triangle's vertices.
    fn face_arc_lengths(&self, triangle: [Vector3D; 3], point: Vector3D) -> ArcLengths;
    // fn face_center(&self, vector1: Vector3D, vector2: Vector3D, vector3: Vector3D) -> Vector3D;
    /// Classic spherical triangle containment test.
    fn is_point_in_face(&self, point: Vector3D, triangle: Vec<Vector3D>) -> bool;
    /// Get angle (in radians) between two unit vectors.
    fn angle_between_unit(&self, u: Vector3D, v: Vector3D) -> f64;
}

#[derive(Default, Debug)]
pub struct ArcLengths {
    pub ab: f64,
    pub bc: f64,
    pub ac: f64,
    pub ap: f64,
    pub bp: f64,
    pub cp: f64,
}
