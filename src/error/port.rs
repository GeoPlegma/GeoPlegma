// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::error::dggal::DggalError;
use crate::error::dggrid::DggridError;
use crate::error::h3o::H3oError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortError {
    #[error("DGGAL error: {0}")]
    Dggal(#[from] DggalError),

    #[error("DGGRID error: {0}")]
    Dggrid(#[from] DggridError),

    #[error("H3o error: {0}")]
    H3o(#[from] H3oError),

    #[error("Unsupported tool/grid combination: {tool}, {grid}")]
    UnsupportedCombo { tool: String, grid: String },
}
