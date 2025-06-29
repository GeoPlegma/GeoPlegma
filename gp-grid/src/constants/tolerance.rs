// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

//! Numerical tolerance constants for geometric calculations
//! 
//! Defines appropriate tolerance values for different types of numerical
//! comparisons and calculations in discrete global grid systems.

/// Default numerical tolerances for various geometric operations
/// 
/// These tolerances are chosen based on the precision requirements
/// of different geometric calculations and the expected numerical
/// errors in floating-point arithmetic.
pub struct Tolerance;

impl Tolerance {
    /// Default tolerance for coordinate comparisons (degrees)
    /// 
    /// Used when comparing geographic coordinates (latitude/longitude) in degrees.
    /// This tolerance of 1e-9 degrees corresponds to approximately 0.1 millimeters
    /// on the Earth's surface, which is appropriate for most DGGRS applications.
    pub const COORDINATE: f64 = 1e-9;
    
    /// Default tolerance for vector normalization operations
    /// 
    /// Used to determine if a vector is too close to zero length to be safely
    /// normalized. This prevents division by zero and numerical instability.
    /// A tolerance of 1e-10 ensures robust behavior while maintaining precision.
    pub const VECTOR_NORMALIZATION: f64 = 1e-10;
    
    /// Default tolerance for angular calculations (radians)
    /// 
    /// Used for comparing angles and trigonometric calculations.
    /// This tolerance of 1e-12 radians corresponds to about 2e-11 degrees,
    /// suitable for high-precision angular computations in projections.
    pub const ANGULAR: f64 = 1e-12;
    
    /// Default tolerance for area calculations
    /// 
    /// Used when comparing areas or determining if geometric shapes
    /// have effectively zero area. Appropriate for spherical triangles
    /// and other area-based calculations in DGGRS.
    pub const AREA: f64 = 1e-12;
    
    /// Default tolerance for distance calculations on unit sphere
    /// 
    /// Used for comparing distances and lengths in unit sphere calculations.
    /// Since the sphere has radius 1, this tolerance is in the same units.
    pub const UNIT_SPHERE_DISTANCE: f64 = 1e-10;
    
    /// Tolerance for barycentric coordinate calculations
    /// 
    /// Used in point-in-triangle tests and other barycentric coordinate
    /// computations. This tolerance accounts for accumulated floating-point
    /// errors in the barycentric coordinate calculation process.
    pub const BARYCENTRIC: f64 = 1e-10;
}

/// Conversion factors between tolerance scales
pub struct ToleranceConversion;

impl ToleranceConversion {
    /// Convert degrees to approximate meters on Earth's surface
    /// 
    /// Uses the authalic radius for conversion. This is an approximation
    /// since the actual distance varies with latitude.
    /// 
    /// Formula: meters ≈ degrees * (π/180) * R_authalic
    pub fn degrees_to_meters(degrees: f64) -> f64 {
        use crate::constants::WGS84;
        degrees * std::f64::consts::PI / 180.0 * WGS84::AUTHALIC_RADIUS
    }
    
    /// Convert meters to approximate degrees on Earth's surface
    /// 
    /// Inverse of degrees_to_meters conversion.
    pub fn meters_to_degrees(meters: f64) -> f64 {
        use crate::constants::WGS84;
        meters * 180.0 / (std::f64::consts::PI * WGS84::AUTHALIC_RADIUS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tolerance_ordering() {
        // More precise operations should have smaller tolerances
        assert!(Tolerance::ANGULAR < Tolerance::COORDINATE);
        assert!(Tolerance::VECTOR_NORMALIZATION < Tolerance::COORDINATE);
    }

    #[test]
    fn test_conversion_roundtrip() {
        let degrees = 1.0;
        let meters = ToleranceConversion::degrees_to_meters(degrees);
        let back_to_degrees = ToleranceConversion::meters_to_degrees(meters);
        
        assert!((degrees - back_to_degrees).abs() < 1e-10);
    }

    #[test]
    fn test_coordinate_tolerance_in_meters() {
        // Verify that coordinate tolerance corresponds to submillimeter precision
        let tolerance_meters = ToleranceConversion::degrees_to_meters(Tolerance::COORDINATE);
        assert!(tolerance_meters < 0.001); // Less than 1 millimeter
        assert!(tolerance_meters > 0.00001); // More than 10 micrometers
    }
}