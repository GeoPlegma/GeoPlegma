use geo_types::Point;
use geoplegma::get;
use geoplegma::models::common::{RefinementLevel, RelativeDepth, ZoneId};

use crate::common::{CONFIG, zone_id_to_u64};
use crate::error::EncodingError;
use crate::storage::StorageBackend;

/// Query a cell value by geographic point.
///
/// The point is resolved to a DGGS cell at `refinement_level`, then the
/// corresponding value bytes are retrieved from the storage backend.
pub fn query_value_bytes_for_point<B: StorageBackend>(
    backend: &B,
    refinement_level: RefinementLevel,
    band: u32,
    point: Point<f64>,
) -> Result<Vec<u8>, EncodingError> {
    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let zones = grid
        .zone_from_point(refinement_level, point, Some(CONFIG))
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

    println!(
        "Resolved point ({}, {}) to zone ID {:?} at level {} and band {}",
        point.x(),
        point.y(),
        zone.id,
        level,
        band
    );

    query_value_bytes_by_cell_index(backend, level, band, &zone.id)
}

/// Query a single cell value by linearized cell index.
///
/// Returns the raw bytes for one attribute value according to the dataset
/// attribute schema.
pub fn query_value_bytes_by_cell_index<B: StorageBackend>(
    backend: &B,
    level: u32,
    band: u32,
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

    let aperture = u64::from(backend.metadata().dggrs.spec().aperture);
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

    let grid = get(backend.metadata().dggrs)
        .map_err(|e| EncodingError::Grid(format!("failed to resolve DGGS: {e}")))?;

    let refinement_level = RefinementLevel::new(i32::try_from(level).map_err(|_| {
        EncodingError::Grid(format!("level {level} cannot be represented as i32"))
    })?)?;

    if refinement_level.get() < depth_i32 {
        return Err(EncodingError::Storage(format!(
            "level {} is below derived chunk depth {}",
            refinement_level.get(),
            depth_i32
        )));
    }

    let chunk_level = RefinementLevel::new(refinement_level.get() - depth_i32)?;

    let mut chunk_zone_id = zone_id.clone();
    for _ in 0..depth_i32 {
        let parent = grid
            .primary_parent_from_zone(chunk_zone_id.clone(), Some(CONFIG))
            .map_err(|e| EncodingError::Grid(e.to_string()))?;
        let parent_zone = parent.zones.first().ok_or_else(|| {
            EncodingError::Grid("primary_parent_from_zone returned no zones".into())
        })?;
        chunk_zone_id = parent_zone.id.clone();
    }

    let chunk_id = zone_id_to_u64(&chunk_zone_id)?;

    let chunk_index = backend
        .metadata()
        .chunk_ids
        .iter()
        .position(|id| *id == chunk_id)
        .ok_or_else(|| {
            EncodingError::Storage(format!(
                "zone {cell_index} belongs to chunk id {chunk_id}, which is not present in dataset metadata"
            ))
        })? as u64;

    let relative_depth = RelativeDepth::new(refinement_level.get() - chunk_level.get())?;
    let children = grid
        .zones_from_parent(relative_depth, chunk_zone_id, Some(CONFIG))
        .map_err(|e| EncodingError::Grid(e.to_string()))?;

    let in_chunk_index = children
        .zones
        .iter()
        .position(|child| child.id == *zone_id)
        .ok_or_else(|| {
            EncodingError::Storage(format!(
                "zone {cell_index} is not present in computed child list for chunk {chunk_id}"
            ))
        })?;

    let chunk = backend.read_chunk(level, band, chunk_index)?;

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

    use geoplegma::get;
    use geoplegma::models::common::{DggrsUid, RefinementLevel, RelativeDepth};

    use crate::common::{CONFIG, zone_id_to_u64};
    use crate::models::{AttributeSchema, DataType, DatasetMetadata};
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

        let dggrs = DggrsUid::H3;
        let grid = get(dggrs).expect("resolve dggrs");

        let min_level = grid.min_refinement_level().expect("min level").get();
        let refinement_level =
            RefinementLevel::new((min_level + 1).max(1)).expect("refinement level");
        let chunk_level = RefinementLevel::new(refinement_level.get() - 1).expect("chunk level");
        let chunk_size = u64::from(dggrs.spec().aperture);

        let chunk_zones = grid
            .zones_from_bbox(chunk_level, None, Some(CONFIG))
            .expect("chunk zones");
        assert!(
            chunk_zones.zones.len() >= 2,
            "expected at least 2 chunk zones"
        );

        let chunk0 = chunk_zones.zones[0].id.clone();
        let chunk1 = chunk_zones.zones[1].id.clone();

        let children0 = grid
            .zones_from_parent(RelativeDepth::new_const(1), chunk0.clone(), Some(CONFIG))
            .expect("children for chunk0");
        let children1 = grid
            .zones_from_parent(RelativeDepth::new_const(1), chunk1.clone(), Some(CONFIG))
            .expect("children for chunk1");

        assert!(
            children0.zones.len() >= 2,
            "expected at least 2 children in chunk0"
        );
        assert!(!children1.zones.is_empty(), "expected children in chunk1");

        let cell0 = children0.zones[0].id.clone();
        let cell1 = children0.zones[1].id.clone();
        let cell2 = children1.zones[0].id.clone();

        let metadata = DatasetMetadata {
            dggrs,
            attributes: vec![AttributeSchema {
                dtype: DataType::UInt16,
                fill_value: None,
            }],
            chunk_size,
            chunk_ids: vec![
                zone_id_to_u64(&chunk0).expect("chunk0 id"),
                zone_id_to_u64(&chunk1).expect("chunk1 id"),
            ],
            levels: vec![refinement_level.get() as u32],
            compression: None,
        };

        let mut backend = ZarrBackend::create(&store_path, metadata).expect("create zarr");
        backend
            .create_level(refinement_level.get() as u32, 0, 2 * chunk_size)
            .expect("create level");

        let mut chunk0_values = vec![0_u16; chunk_size as usize];
        chunk0_values[0] = 10;
        chunk0_values[1] = 20;

        let mut chunk1_values = vec![0_u16; chunk_size as usize];
        chunk1_values[0] = 60;

        let chunk0_bytes: Vec<u8> = chunk0_values
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();
        let chunk1_bytes: Vec<u8> = chunk1_values
            .into_iter()
            .flat_map(u16::to_ne_bytes)
            .collect();

        backend
            .write_chunk(refinement_level.get() as u32, 0, 0, &chunk0_bytes)
            .expect("write chunk0");
        backend
            .write_chunk(refinement_level.get() as u32, 0, 1, &chunk1_bytes)
            .expect("write chunk1");

        let bytes =
            query_value_bytes_by_cell_index(&backend, refinement_level.get() as u32, 0, &cell0)
                .expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 10);

        let bytes =
            query_value_bytes_by_cell_index(&backend, refinement_level.get() as u32, 0, &cell1)
                .expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 20);

        let bytes =
            query_value_bytes_by_cell_index(&backend, refinement_level.get() as u32, 0, &cell2)
                .expect("query cell");
        let value = u16::from_ne_bytes([bytes[0], bytes[1]]);
        assert_eq!(value, 60);

        let _ = std::fs::remove_dir_all(&store_path);
    }
}
