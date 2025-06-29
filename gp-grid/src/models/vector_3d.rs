// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

use std::ops::{Add, Sub, Mul, Neg};

/// A 3D vector optimized for geometric operations on the unit sphere.
/// Primarily used for discrete global grid system calculations.
/// 
/// **Design Reasoning**: 
/// - Uses f64 for precision in spherical calculations
/// - Focuses on operations needed for icosahedron/DGGRS computations
#[derive(Clone, Copy, Debug)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    /// Create a new 3D vector
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Zero vector constant
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Convert to array [x, y, z]
    pub const fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Compute squared magnitude (avoids expensive sqrt)
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Check if vector is approximately zero
    pub fn is_zero(self, tolerance: f64) -> bool {
        self.length_squared() < tolerance * tolerance
    }

    /// Safe normalization that returns None for zero-length vectors
    pub fn try_normalize(self, tolerance: f64) -> Option<Self> {
        let length_sq = self.length_squared();
        if length_sq < tolerance * tolerance {
            None
        } else {
            let inv_length = 1.0 / length_sq.sqrt();
            Some(Self {
                x: self.x * inv_length,
                y: self.y * inv_length,
                z: self.z * inv_length,
            })
        }
    }

    /// Vector addition
    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Scalar multiplication
    pub fn scale(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
    pub fn cross(self, other: Self) -> Vector3D {
        Vector3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn from_array(array: [f64; 3]) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn mid(a: Self, b: Self) -> Vector3D {
        Vector3D {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
            z: (a.z + b.z) / 2.0,
        }
    }
    pub fn neg(self) -> Vector3D {
        Vector3D {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
    pub fn normalize(self) -> Vector3D {
        self.try_normalize(1e-10)
            .expect("Cannot normalize zero-length vector")
    }

    pub fn subtract(self, other: Vector3D) -> Vector3D {
        Vector3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

// Standard operator trait implementations for better ergonomics
impl Add for Vector3D {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector3D {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f64> for Vector3D {
    type Output = Self;
    
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vector3D> for f64 {
    type Output = Vector3D;
    
    fn mul(self, vector: Vector3D) -> Vector3D {
        vector * self
    }
}

impl Neg for Vector3D {
    type Output = Self;
    
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Default for Vector3D {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::from_array([1.0, 2.0, 3.0]);
        
        assert_eq!(v1.x, 1.0);
        assert_eq!(v1.y, 2.0);
        assert_eq!(v1.z, 3.0);
        assert_eq!(v1.to_array(), [1.0, 2.0, 3.0]);
        assert_eq!(v2.x, v1.x);
    }

    #[test]
    fn test_arithmetic_operators() {
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::new(4.0, 5.0, 6.0);
        
        let sum = v1 + v2;
        assert_eq!(sum.x, 5.0);
        assert_eq!(sum.y, 7.0);
        assert_eq!(sum.z, 9.0);
        
        let diff = v2 - v1;
        assert_eq!(diff.x, 3.0);
        assert_eq!(diff.y, 3.0);
        assert_eq!(diff.z, 3.0);
        
        let scaled = v1 * 2.0;
        assert_eq!(scaled.x, 2.0);
        assert_eq!(scaled.y, 4.0);
        assert_eq!(scaled.z, 6.0);
        
        let neg = -v1;
        assert_eq!(neg.x, -1.0);
        assert_eq!(neg.y, -2.0);
        assert_eq!(neg.z, -3.0);
    }

    #[test]
    fn test_length_and_normalization() {
        let v = Vector3D::new(3.0, 4.0, 0.0);
        assert_eq!(v.length_squared(), 25.0);
        assert_eq!(v.length(), 5.0);
        
        let normalized = v.normalize();
        assert!((normalized.length() - 1.0).abs() < 1e-10);
        
        // Test zero vector safety
        let zero = Vector3D::zero();
        assert!(zero.is_zero(1e-10));
        assert!(zero.try_normalize(1e-10).is_none());
    }

    #[test]
    #[should_panic(expected = "Cannot normalize zero-length vector")]
    fn test_normalize_zero_panic() {
        let zero = Vector3D::zero();
        let _ = zero.normalize();
    }

    #[test]
    fn test_dot_and_cross_product() {
        let v1 = Vector3D::new(1.0, 0.0, 0.0);
        let v2 = Vector3D::new(0.0, 1.0, 0.0);
        
        assert_eq!(v1.dot(v2), 0.0);
        
        let cross = v1.cross(v2);
        assert_eq!(cross.x, 0.0);
        assert_eq!(cross.y, 0.0);
        assert_eq!(cross.z, 1.0);
    }

    #[test]
    fn test_backward_compatibility() {
        let v1 = Vector3D::new(1.0, 2.0, 3.0);
        let v2 = Vector3D::new(4.0, 5.0, 6.0);
        
        // Test that old method interfaces still work
        let mid = Vector3D::mid(v1, v2);
        assert_eq!(mid.x, 2.5);
        assert_eq!(mid.y, 3.5);
        assert_eq!(mid.z, 4.5);
        
        let sub = v1.subtract(v2);
        assert_eq!(sub.x, -3.0);
        assert_eq!(sub.y, -3.0);
        assert_eq!(sub.z, -3.0);
        
        let neg = v1.neg();
        assert_eq!(neg.x, -1.0);
        assert_eq!(neg.y, -2.0);
        assert_eq!(neg.z, -3.0);
    }
}
