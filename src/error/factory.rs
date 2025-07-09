use std::num::TryFromIntError;
use thiserror::Error;

/// Error type for instantiating DggrsPort adapters via the factory.
#[derive(Debug, Error)]
pub enum FactoryError {
    #[error("Unsupported combination: tool='{tool}', dggrs='{dggrs}'")]
    UnsupportedCombination { tool: String, dggrs: String },
}
