use std::path::Path;

use crate::error::EncodingError;
use crate::models::DatasetMetadata;

pub trait LevelHandle: Send + Sync {}
pub trait StorageBackend: Sized + Send + Sync {
    type Level: LevelHandle;

    fn create(path: &Path, metadata: DatasetMetadata) -> Result<Self, EncodingError>;

    fn open(path: &Path) -> Result<Self, EncodingError>;

    fn metadata(&self) -> &DatasetMetadata;

    fn create_level(&mut self, level: u32, band: u32, num_cells: u64) -> Result<Self::Level, EncodingError>;

    fn levels(&self) -> Vec<u32>;
    fn band_count(&self) -> u32;

    fn write_chunk(&self, level: u32, band: u32, chunk_index: u64, data: &[u8]) -> Result<(), EncodingError>;

    fn read_chunk(&self, level: u32, band: u32, chunk_index: u64) -> Result<Vec<u8>, EncodingError>;

    fn num_chunks(&self, level: u32) -> Result<u64, EncodingError>;

}
