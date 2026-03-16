use std::collections::BTreeMap;
use std::path::Path;

use geoplegma::models::common::RefinementLevel;
use gdal::{Dataset, GeoTransformEx};
use geo_types::Point;
use rayon::prelude::*;

use crate::error::EncodingError;
use crate::grid::Linearizer;
use crate::models::{DataType, DatasetMetadata, GridExtent};
use crate::storage::StorageBackend;

pub fn convert_geotiff_file_to_backend<B, G>(
    geotiff_path: &Path,
    output_path: &Path,
    grid: &G,
    refinement_level: RefinementLevel,
    sample: usize,
    mut metadata: DatasetMetadata,
) -> Result<B, EncodingError>
where
    B: StorageBackend,
    G: Linearizer,
{
    if metadata.attributes.is_empty() {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one attribute".into(),
        ));
    }

    let level = u32::try_from(refinement_level.get()).map_err(|_| {
        EncodingError::Grid(format!(
            "refinement level {} cannot be represented as u32",
            refinement_level.get()
        ))
    })?;

    let dataset = Dataset::open(geotiff_path).map_err(|e| EncodingError::Gdal(e.to_string()))?;

    if sample >= dataset.raster_count() {
        return Err(EncodingError::Gdal(format!(
            "sample index {sample} out of range; dataset has {} raster bands",
            dataset.raster_count()
        )));
    }

    let band = dataset
        .rasterband(sample + 1)
        .map_err(|e| EncodingError::Gdal(e.to_string()))?;

    let (width, height) = band.size();

    if width == 0 || height == 0 {
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let gt = dataset
        .geo_transform()
        .map_err(|e| EncodingError::Gdal(format!("missing/invalid geotransform: {e}")))?;

    let corners = [
        gt.apply(0.0, 0.0),
        gt.apply(width as f64, 0.0),
        gt.apply(0.0, height as f64),
        gt.apply(width as f64, height as f64),
    ];

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for (x, y) in corners {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    metadata.extent = GridExtent::BoundingBox {
        min_lon: min_x,
        min_lat: min_y,
        max_lon: max_x,
        max_lat: max_y,
    };

    let output_type = metadata.attributes[0].dtype;
    let value_size = output_type.byte_size();

    let raster = band
        .read_band_as::<f64>()
        .map_err(|e| EncodingError::Gdal(e.to_string()))?;
    let data = raster.data();

    let mut linear_values: BTreeMap<u64, Vec<u8>> = BTreeMap::new();

    let total_pixels = height * width;

    let computed_entries: Result<Vec<_>, EncodingError> = (0..total_pixels)
        .into_par_iter()
        .map(|idx| {
            let row = idx / width;
            let col = idx % width;

            let v = *data
                .get(idx)
                .ok_or_else(|| EncodingError::Gdal("raster buffer index out of bounds".into()))?;

            // pixel center
            let (x, y) = gt.apply(col as f64 + 0.5, row as f64 + 0.5);

            let zones = grid
                .zone_from_point(refinement_level, Point::new(x, y), None)
                .map_err(|e| EncodingError::Grid(e.to_string()))?;

            let zone = zones
                .zones
                .first()
                .ok_or_else(|| EncodingError::Grid("zone_from_point returned no zones".into()))?;

            let bytes = f64_to_bytes(v, output_type)?;
            let key = grid.zone_to_linear(&zone.id);

            Ok((key, bytes))
        })
        .collect();

    linear_values.extend(computed_entries?);

    let num_cells = grid.num_cells_at_level(refinement_level);
    let chunk_size = metadata.chunk_size;

    let mut backend = B::create(output_path, metadata)?;
    backend.create_level(level, num_cells)?;

    let mut chunks: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    for (linear_index, value_bytes) in linear_values {
        let chunk_index = linear_index / chunk_size;
        let in_chunk_index = (linear_index % chunk_size) as usize;

        let chunk_buf = chunks
            .entry(chunk_index)
            .or_insert_with(|| vec![0_u8; chunk_size as usize * value_size]);

        let start = in_chunk_index * value_size;
        let end = start + value_size;
        chunk_buf[start..end].copy_from_slice(&value_bytes);
    }

    for (chunk_index, bytes) in chunks {
        backend.write_chunk(level, chunk_index, &bytes)?;
    }

    Ok(backend)
}

fn f64_to_bytes(value: f64, data_type: DataType) -> Result<Vec<u8>, EncodingError> {
    match data_type {
        DataType::Float32 => Ok((value as f32).to_ne_bytes().to_vec()),
        DataType::Float64 => Ok(value.to_ne_bytes().to_vec()),
        DataType::Int8 => cast_int::<i8>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::Int16 => cast_int::<i16>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::Int32 => cast_int::<i32>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::Int64 => cast_int::<i64>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::UInt8 => cast_uint::<u8>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::UInt16 => cast_uint::<u16>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::UInt32 => cast_uint::<u32>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
        DataType::UInt64 => cast_uint::<u64>(value)
            .map(|v| v.to_ne_bytes().to_vec())
            .map_err(EncodingError::GeoTiff),
    }
}

fn cast_int<T>(value: f64) -> Result<T, String>
where
    T: TryFrom<i64>,
{
    if value.fract() != 0.0 {
        return Err(format!(
            "non-integer GeoTIFF value {value} for integer output type"
        ));
    }

    let as_i64 = value as i64;
    T::try_from(as_i64).map_err(|_| format!("GeoTIFF value {value} out of range"))
}

fn cast_uint<T>(value: f64) -> Result<T, String>
where
    T: TryFrom<u64>,
{
    if value < 0.0 || value.fract() != 0.0 {
        return Err(format!(
            "GeoTIFF value {value} is invalid for unsigned integer output type"
        ));
    }

    let as_u64 = value as u64;
    T::try_from(as_u64).map_err(|_| format!("GeoTIFF value {value} out of range"))
}
