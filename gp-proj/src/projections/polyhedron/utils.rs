// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
// Modified by Sunayana Ghosh (sunayanag@gmail.com)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use crate::models::vector_3d::Vector3D;

/// Uses barycentric coordinates to test if point in in spherical triangle
pub fn is_point_in_face(point: Vector3D, triangle: &[Vector3D]) -> bool {
    if triangle.len() != 3 {
        return false;
    }

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
        return false; //Degenerate triangle
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    // Point is in triangle if all barycentric coordinates are non-negative
    // Use small tolerance for numerical stability
    const TOLERANCE: f64 = 1e-10;
    u >= -TOLERANCE && v >= -TOLERANCE && (u + v) <= 1.0 + TOLERANCE
}

/// Numerically stable angle between two unit vectors
/// Uses atan2 method for better numerical stability than acos
/// **Return the angle between unit vectors**
pub fn angle_between_unit(u: Vector3D, v: Vector3D) -> f64 {
    // For unit vectors, use the cross product magnitude and dot product
    // with atan2 for numerical stability
    let cross = u.cross(v);
    let cross_magnitude = cross.length();
    let dot = u.dot(v);

    // atan2 handles all quadrants correctly and is more stable than acos
    cross_magnitude.atan2(dot)
}
