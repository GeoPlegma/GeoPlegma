// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::models::vector_3d::Vector3D;

pub trait Polyhedron {
    /// Return the actual 3D vertices of each face.
    fn vertices(&self) -> Vec<Vector3D>;

    /// Return index triplets of the icosahedron faces.
    fn face_vertex_indices(&self) -> Vec<Face>;

    /// Compute the centroid of a triangle face.
    fn face_center(&self, face_id: usize) -> Vector3D;

    /// Given a point on the unit sphere, return the face index that contains it.
    fn find_face(&self, point: Vector3D) -> Option<usize>;

    /// Compute spherical arc lengths between point P and the triangle's vertices.
    fn face_arc_lengths(&self, triangle: [Vector3D; 3], point: Vector3D) -> ArcLengths;

    /// Classic spherical triangle containment test.
    fn is_point_in_face(&self, point: Vector3D, face: Vec<Vector3D>) -> bool;

    /// Get angle (in radians) between two unit vectors.
    fn angle_between_unit(&self, u: Vector3D, v: Vector3D) -> f64;
}

pub enum Face {
    Triangle([usize; 3]),
    Quad([usize; 4]),
    Pentagon([usize; 5]),
    Hexagon([usize; 6]),
    Polygon(Vec<usize>), // for rare or irregular faces
}

impl Face {
    pub fn indices(&self) -> &[usize] {
        match self {
            Face::Triangle(v) => v,
            Face::Quad(v) => v,
            Face::Pentagon(v) => v,
            Face::Hexagon(v) => v,
            Face::Polygon(v) => v,
        }
    }
}

#[derive(Default)]
pub struct ArcLengths {
    pub ab: f64,
    pub bc: f64,
    pub ac: f64,
    pub ap: f64,
    pub bp: f64,
    pub cp: f64,
}
