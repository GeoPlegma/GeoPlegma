// Copyright 2025 contributors to the GeoPlegmata project.
// Originally authored by Michael Jendryke, GeoInsight (michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use thiserror::Error;

use crate::models::common::{DggrsTool, DggrsUid};

/// Error type for instantiating DggrsPort adapters via the factory.
#[derive(Debug, Error)]
pub enum FactoryError {
    #[error("Unsupported combination: tool='{tool}', dggrs='{id}'")]
    UnsupportedCombination { tool: DggrsTool, id: DggrsUid },
    #[error("DGGRS ID {id} is not available")]
    AdapterUnavailable { id: DggrsUid },
}
