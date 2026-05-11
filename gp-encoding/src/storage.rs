use std::path::Path;

use crate::error::EncodingError;
use crate::models::DatasetMetadata;

pub trait LevelHandle: Send + Sync {}
pub trait StorageBackend: Sized + Send + Sync {
    type Level: LevelHandle;

    fn create(path: &Path, metadata: DatasetMetadata) -> Result<Self, EncodingError>;

    fn open(path: &Path) -> Result<Self, EncodingError>;

    fn metadata(&self) -> &DatasetMetadata;

    fn create_level(
        &mut self,
        level: u32,
        band: u32,
        num_cells: u64,
    ) -> Result<Self::Level, EncodingError>;

    fn levels(&self) -> Vec<u32>;
    fn band_count(&self) -> u32;

    fn write_chunk(
        &self,
        level: u32,
        band: u32,
        chunk_index: u64,
        data: &[u8],
    ) -> Result<(), EncodingError>;

    fn read_chunk(&self, level: u32, band: u32, chunk_index: u64)
    -> Result<Vec<u8>, EncodingError>;

    fn num_chunks(&self, level: u32) -> Result<u64, EncodingError>;

    fn chunk_ids_for_level(&self, level: u32) -> Result<Vec<String>, EncodingError>;

    fn set_level_chunk_ids(
        &mut self,
        level: u32,
        chunk_ids: Vec<String>,
    ) -> Result<(), EncodingError>;
}

pub fn compute_chunk_depth(chunk_size: u64, aperture: u64) -> Result<i32, EncodingError> {
    if aperture <= 1 {
        return Err(EncodingError::Storage(format!(
            "invalid DGGS aperture {aperture} for chunk resolution"
        )));
    }

    let mut depth_i32 = 0_i32;
    let mut size_check = 1_u64;
    while size_check < chunk_size {
        size_check = size_check.checked_mul(aperture).ok_or_else(|| {
            EncodingError::Storage("chunk_size power computation overflow".into())
        })?;
        depth_i32 += 1;
    }

    if size_check != chunk_size {
        return Err(EncodingError::Storage(format!(
            "chunk_size {chunk_size} is not a power of aperture {aperture}"
        )));
    }

    Ok(depth_i32)
}
