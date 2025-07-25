// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms

//! Constants and parameters for discrete global grid systems
//! 
//! This module provides well-organized constants for:
//! - Earth ellipsoid parameters (WGS84)
//! - Projection transformation coefficients
//! - Numerical tolerances for calculations

pub mod earth;
pub mod polyhedron;
pub mod projections;
pub mod tolerance;

// Re-export commonly used constants for convenience
pub use earth::WGS84;
pub use polyhedron::PolyhedronConstants;
pub use projections::{KarneyCoefficients, IcosahedronConstants};
pub use tolerance::Tolerance;