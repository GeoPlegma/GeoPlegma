// Copyright 2025 contributors to the GeoPlegmata project.
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utility functions for geometric and geographic operations
//! 
//! This module provides helper functions that operate on coordinates,
//! vectors, and other geometric primitives used throughout the DGGRS system.

pub mod geo;

// Re-export commonly used utilities
pub use geo::*;