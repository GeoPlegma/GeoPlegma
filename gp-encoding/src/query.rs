use geo_types::Point;
use geoplegma::get;
use geoplegma::models::common::{RefinementLevel, ZoneId};

use crate::common::zone_id_to_u64;
use crate::error::EncodingError;
use crate::storage::StorageBackend;

/// Query a cell value by geographic point.
///
/// The point is resolved to a DGGS cell at `refinement_level`, then the
/// corresponding value bytes are retrieved from the storage backend.
pub fn query_value_bytes_for_point<B: StorageBackend>(
    backend: &B,
    refinement_level: RefinementLevel,
    point: Point<f64>,
) -> Result<Vec<u8>, EncodingError> {
    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let zones = grid
        .zone_from_point(refinement_level, point, None)
        .map_err(|e| EncodingError::Grid(e.to_string()))?;

    let zone = zones
        .zones
        .first()
        .ok_or_else(|| EncodingError::Grid("zone_from_point returned no zones".into()))?;

    let level = u32::try_from(refinement_level.get()).map_err(|_| {
        EncodingError::Grid(format!(
            "refinement level {} cannot be represented as u32",
            refinement_level.get()
        ))
    })?;

    query_value_bytes_by_cell_index(backend, level, &zone.id)
}

/// Query a single cell value by linearized cell index.
///
/// Returns the raw bytes for one attribute value according to the dataset
/// attribute schema.
pub fn query_value_bytes_by_cell_index<B: StorageBackend>(
    backend: &B,
    level: u32,
    zone_id: &ZoneId,
) -> Result<Vec<u8>, EncodingError> {
    let cell_index = zone_id_to_u64(zone_id)?;
    let value_size = backend
        .metadata()
        .attributes
        .first()
        .map(|attr| attr.dtype.byte_size())
        .ok_or_else(|| {
            EncodingError::Storage("dataset metadata must define at least one attribute".into())
        })?;

    let chunk_size = backend.metadata().chunk_size;
    if chunk_size == 0 {
        return Err(EncodingError::Storage(
            "dataset metadata chunk_size must be greater than zero".into(),
        ));
    }

    let chunk_index = cell_index / chunk_size;
    let in_chunk_index = (cell_index % chunk_size) as usize;

    let chunk = backend.read_chunk(level, chunk_index)?;

    let start = in_chunk_index * value_size;
    let end = start + value_size;
    if chunk.len() < end {
        return Err(EncodingError::Storage(format!(
            "chunk {chunk_index} at level {level} is too small for cell index {cell_index}"
        )));
    }

    Ok(chunk[start..end].to_vec())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use geoplegma::models::common::{DggrsUid, ZoneId};

    use crate::models::{AttributeSchema, DataType, DatasetMetadata, GridExtent};
    use crate::query::query_value_bytes_by_cell_index;
    use crate::storage::StorageBackend;
    use crate::zarr::ZarrBackend;

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("gp_encoding_{name}_{nanos}"))
    }

    #[test]
    fn query_value_bytes_by_cell_index_reads_expected_cell() {
        let store_path = unique_temp_dir("query_test");

        let metadata = DatasetMetadata {
            dggrs: DggrsUid::H3,
            extent: GridExtent::Global,
            attributes: vec![AttributeSchema {
                name: "value".to_string(),
                dtype: DataType::UInt16,
                fill_value: None,
            }],
            chunk_size: 4,
            levels: vec![1],
            compression: None,
        };

        let mut backend = ZarrBackend::create(&store_path, metadata).expect("create zarr");
        backend.create_level(1, 8).expect("create level");

        let chunk0: Vec<u8> = [10_u16, 20, 30, 40]
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();
        let chunk1: Vec<u8> = [50_u16, 60, 70, 80]
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();

        backend.write_chunk(1, 0, &chunk0).expect("write chunk0");
        backend.write_chunk(1, 1, &chunk1).expect("write chunk1");

        let bytes = query_value_bytes_by_cell_index(&backend, 1, &ZoneId::IntId(0)).expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 10);

        let bytes = query_value_bytes_by_cell_index(&backend, 1, &ZoneId::IntId(1)).expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 20);


        let bytes = query_value_bytes_by_cell_index(&backend, 1, &ZoneId::IntId(5)).expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 60);

        let _ = std::fs::remove_dir_all(&store_path);
    }
}