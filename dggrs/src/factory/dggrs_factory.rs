// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::adapters::{
    dggal::grids::DggalImpl, dggrid::igeo7::Igeo7Impl, dggrid::isea3h::Isea3hImpl, h3o::h3::H3Impl,
};
use crate::error::factory::FactoryError;
use crate::ports::dggrs::DggrsPort;
use std::sync::Arc;

pub fn get(tool: &str, dggrs: &str) -> Result<Arc<dyn DggrsPort>, FactoryError> {
    match (tool.to_uppercase().as_str(), dggrs.to_uppercase().as_str()) {
        ("DGGRID", "ISEA3H") => Ok(Arc::new(Isea3hImpl::default())),
        ("DGGRID", "IGEO7") => Ok(Arc::new(Igeo7Impl::default())),
        ("H3O", "H3") => Ok(Arc::new(H3Impl::default())),
        ("DGGAL", "IVEA3H") => Ok(Arc::new(DggalImpl::new("IVEA3H"))),
        ("DGGAL", "IVEA9R") => Ok(Arc::new(DggalImpl::new("IVEA9R"))),
        ("DGGAL", "ISEA3H") => Ok(Arc::new(DggalImpl::new("ISEA3H"))),
        ("DGGAL", "ISEA9R") => Ok(Arc::new(DggalImpl::new("ISEA9R"))),
        ("DGGAL", "RTEA3H") => Ok(Arc::new(DggalImpl::new("RTEA3H"))),
        ("DGGAL", "RTEA9R") => Ok(Arc::new(DggalImpl::new("RTEA9R"))),
        _ => Err(FactoryError::UnsupportedCombination {
            tool: tool.to_string(),
            dggrs: dggrs.to_string(),
        }),
    }
}
