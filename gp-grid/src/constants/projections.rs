// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

//! Projection transformation coefficients and constants
//! 
//! Contains mathematical coefficients and parameters used in map projections
//! and coordinate transformations, particularly for discrete global grid systems.

/// Karney latitude transformation coefficients (2023)
/// 
/// These coefficients are used for high-precision conversions between
/// geodetic and authalic latitudes as described in:
/// "Transverse Mercator with an accuracy of a few nanometers" by Charles F.F. Karney
/// https://arxiv.org/pdf/2212.05818
pub struct KarneyCoefficients;

impl KarneyCoefficients {
    /// Coefficients to convert geodetic latitude to authalic latitude
    /// 
    /// Cξφ coefficients from equation A19 in Karney (2023).
    /// Used to transform from the ellipsoidal (geodetic) latitude to the 
    /// authalic latitude on a sphere of equal surface area.
    /// 
    /// The transformation uses Clenshaw summation with these Fourier coefficients
    /// computed using the Horner method for numerical stability.
    pub const GEODETIC_TO_AUTHALIC: [f64; 21] = [
        4.0 / 3.0,
        4.0 / 45.0,
        -16.0 / 35.0,
        -2582.0 / 14175.0,
        60136.0 / 467775.0,
        28112932.0 / 212837625.0,
        46.0 / 45.0,
        152.0 / 945.0,
        -11966.0 / 14175.0,
        -21016.0 / 51975.0,
        251310128.0 / 638512875.0,
        3044.0 / 2835.0,
        3802.0 / 14175.0,
        -94388.0 / 66825.0,
        -8797648.0 / 10945935.0,
        6059.0 / 4725.0,
        41072.0 / 93555.0,
        -1472637812.0 / 638512875.0,
        768272.0 / 467775.0,
        -455935736.0 / 638512875.0,
        4210684958.0 / 1915538625.0,
    ];
    
    /// Coefficients to convert authalic latitude to geodetic latitude
    /// 
    /// Cφξ coefficients from equation A20 in Karney (2023).
    /// Used for the inverse transformation from authalic latitude back to
    /// geodetic latitude on the WGS84 ellipsoid.
    /// 
    /// These coefficients enable high-precision reverse transformations
    /// with accuracy to the nanometer level.
    pub const AUTHALIC_TO_GEODETIC: [f64; 21] = [
        -4.0 / 3.0,
        -4.0 / 45.0,
        88.0 / 315.0,
        538.0 / 4725.0,
        20824.0 / 467775.0,
        -44732.0 / 2837835.0,
        34.0 / 45.0,
        8.0 / 105.0,
        -2482.0 / 14175.0,
        -37192.0 / 467775.0,
        -12467764.0 / 212837625.0,
        -1532.0 / 2835.0,
        -898.0 / 14175.0,
        54968.0 / 467775.0,
        100320856.0 / 1915538625.0,
        6007.0 / 14175.0,
        24496.0 / 467775.0,
        -5884124.0 / 70945875.0,
        -23356.0 / 66825.0,
        -839792.0 / 19348875.0,
        570284222.0 / 1915538625.0,
    ];
}

/// Constants specific to icosahedral projections
/// 
/// The icosahedron is a regular polyhedron with 20 triangular faces,
/// commonly used as the base polyhedron for discrete global grid systems
/// due to its near-spherical shape and good area/shape distortion properties.
pub struct IcosahedronConstants;

impl IcosahedronConstants {
    /// Radius ratio for icosahedral projection
    /// 
    /// This ratio relates the radius of the circumscribed sphere to the
    /// effective projection radius in icosahedral DGGRS implementations.
    /// 
    /// Derivation: RR = R' / R = (1 / (2 * sqrt(5)) + 1 / 6) * sqrt(PI * sqrt(3))
    /// 
    /// This constant is used in the Vertex Great Circle (VGC) projection
    /// and other icosahedron-based discrete global grid systems.
    pub const RADIUS_RATIO: f64 = 0.9103832815095034;
    
    /// Number of faces in an icosahedron
    pub const FACE_COUNT: u8 = 20;
    
    /// Number of vertices in an icosahedron  
    pub const VERTEX_COUNT: u8 = 12;
    
    /// Number of edges in an icosahedron
    pub const EDGE_COUNT: u8 = 30;
    
    /// Dihedral angle between adjacent faces (radians)
    /// 
    /// The angle between two adjacent triangular faces of a regular icosahedron.
    /// This is approximately 138.19° or 2.4124 radians.
    /// Calculated as: π - arccos(2π/5) ≈ 2.41231732
    pub const DIHEDRAL_ANGLE: f64 = 2.41231732;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_karney_coefficients_length() {
        // Both coefficient arrays should have exactly 21 elements
        assert_eq!(KarneyCoefficients::GEODETIC_TO_AUTHALIC.len(), 21);
        assert_eq!(KarneyCoefficients::AUTHALIC_TO_GEODETIC.len(), 21);
    }

    #[test]
    fn test_icosahedron_euler_characteristic() {
        // Verify Euler's formula for polyhedra: V - E + F = 2
        let v = IcosahedronConstants::VERTEX_COUNT as i32;
        let e = IcosahedronConstants::EDGE_COUNT as i32;
        let f = IcosahedronConstants::FACE_COUNT as i32;
        
        assert_eq!(v - e + f, 2);
    }

    #[test]
    fn test_radius_ratio_bounds() {
        // Radius ratio should be reasonable (between 0 and 1)
        assert!(IcosahedronConstants::RADIUS_RATIO > 0.0);
        assert!(IcosahedronConstants::RADIUS_RATIO < 1.0);
    }
}