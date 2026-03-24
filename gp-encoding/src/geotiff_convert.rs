use std::collections::BTreeMap;
use std::path::Path;

use gdal::raster::GdalDataType;
use gdal::{Dataset, GeoTransformEx};
use geo_types::Point;
use geoplegma::api::DggrsApi;
use geoplegma::get;
use geoplegma::models::common::{DggrsUid, RefinementLevel};
use rayon::prelude::*;

use crate::AttributeSchema;
use crate::common::{CONFIG, zone_id_to_u64};
use crate::error::EncodingError;
use crate::models::{DataType, DatasetMetadata, GridExtent};
use crate::storage::StorageBackend;

trait NativeBytes {
    fn to_native_bytes(self) -> Vec<u8>;
}

macro_rules! impl_native_bytes {
    ($($t:ty),+ $(,)?) => {
        $(
            impl NativeBytes for $t {
                fn to_native_bytes(self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }
            }
        )+
    };
}

impl_native_bytes!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

fn compute_entries_from_data<T>(
    data: &[T],
    total_pixels: usize,
    width: usize,
    gt: [f64; 6],
    grid: &dyn DggrsApi,
    refinement_level: RefinementLevel,
) -> Result<Vec<(u64, Vec<u8>)>, EncodingError>
where
    T: NativeBytes + Copy + Send + Sync,
{
    (0..total_pixels)
        .into_par_iter()
        .map(|idx| {
            let row = idx / width;
            let col = idx % width;

            let v = *data.get(idx).ok_or_else(|| {
                EncodingError::GeoTiff(format!(
                    "index {idx} out of bounds for data length {}",
                    data.len()
                ))
            })?;

            // pixel center
            let (x, y) = gt.apply(col as f64 + 0.5, row as f64 + 0.5);

            let zones = grid.zone_from_point(refinement_level, Point::new(x, y), Some(CONFIG))?;

            let zone = zones
                .zones
                .first()
                .ok_or_else(|| EncodingError::Grid("zone_from_point returned no zones".into()))?;

            let key = zone_id_to_u64(&zone.id)?;
            Ok((key, v.to_native_bytes()))
        })
        .collect()
}

fn get_closest_refinement_level(
    grid: &std::sync::Arc<dyn geoplegma::api::DggrsApi>,
    gt: [f64; 6],
) -> Result<RefinementLevel, EncodingError> {
    let pixel_width = gt[1].abs();
    let pixel_height = gt[5].abs();

    if pixel_width == 0.0 || pixel_height == 0.0 {
        return Err(EncodingError::GeoTiff(
            "geotransform has zero pixel size".into(),
        ));
    }

    let world_pixel_count =
        ((40_075_016.685 / pixel_width) * (40_075_016.685 / pixel_height)) as u64; // TODO: support different projections?

    println!("world pixel count: {}", world_pixel_count);
    let mut best_level: Option<RefinementLevel> = None;
    let mut best_diff = u64::MAX;

    let min_level = grid.min_refinement_level()?;
    let max_level = grid.max_refinement_level()?;

    for raw_level in min_level.get()..=max_level.get() {
        let level = RefinementLevel::new_const(raw_level);

        let zone_count = grid.zone_count(level)?;

        let diff = world_pixel_count.abs_diff(zone_count);

        if diff < best_diff {
            best_diff = diff;
            best_level = Some(level);
        }
    }
    println!(
        "best level: {} with diff {}",
        best_level.unwrap().get(),
        best_diff
    );

    best_level.ok_or_else(|| EncodingError::Grid("no valid refinement level found".into()))
}

pub fn convert_geotiff_file_to_backend<B>(
    geotiff_path: &Path,
    output_path: &Path,
    dggrs: DggrsUid,
) -> Result<B, EncodingError>
where
    B: StorageBackend,
{
    let grid = get(dggrs)?;

    let dataset = Dataset::open(geotiff_path)?;

    let bands = dataset
        .rasterbands()
        .map(|b| b.map(|band| band.band_type()))
        .collect::<Result<Vec<_>, _>>()?;
    let metadata_bands = bands
        .iter()
        .map(|band_type| {
            let dtype = match band_type {
                GdalDataType::UInt8 => DataType::UInt8,
                GdalDataType::Int8 => DataType::Int8,
                GdalDataType::Int16 => DataType::Int16,
                GdalDataType::UInt16 => DataType::UInt16,
                GdalDataType::Int32 => DataType::Int32,
                GdalDataType::UInt32 => DataType::UInt32,
                GdalDataType::Int64 => DataType::Int64,
                GdalDataType::UInt64 => DataType::UInt64,
                GdalDataType::Float32 => DataType::Float32,
                GdalDataType::Float64 => DataType::Float64,
                // TODO: add every type
                _ => {
                    return Err(EncodingError::GeoTiff(format!(
                        "unsupported GDAL data type: {band_type:?}"
                    )));
                }
            };

            Ok(AttributeSchema {
                dtype,
                fill_value: Some("0.0".to_string()),
            })
        })
        .collect::<Result<Vec<_>, EncodingError>>()?;

    if metadata_bands.is_empty() {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one attribute".into(),
        ));
    }

    let first_band = dataset.rasterband(1)?;

    let (width, height) = first_band.size();

    if width == 0 || height == 0 {
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let total_pixels = height * width;

    let gt = dataset.geo_transform()?;

    let refinement_level = get_closest_refinement_level(&grid, gt)?;

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

    let metadata = DatasetMetadata {
        dggrs,
        extent: GridExtent::BoundingBox {
            min_lon: min_x,
            min_lat: min_y,
            max_lon: max_x,
            max_lat: max_y,
        },
        attributes: metadata_bands,
        chunk_size: 1024,
        levels: vec![refinement_level.get() as u32],
        compression: None,
    };

    let zones = grid.zone_count(refinement_level)?;
    let chunk_size = metadata.chunk_size;
    let mut backend = B::create(output_path, metadata)?;

    for (band_index, band_type) in bands.into_iter().enumerate() {
        let band = dataset.rasterband(band_index + 1)?;
        let value_size = band_type.bytes() as usize;

        let computed_entries = match band_type {
            GdalDataType::UInt8 => {
                let raster = band.read_band_as::<u8>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::Int8 => {
                let raster = band.read_band_as::<i8>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::UInt16 => {
                let raster = band.read_band_as::<u16>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::Int16 => {
                let raster = band.read_band_as::<i16>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::UInt32 => {
                let raster = band.read_band_as::<u32>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::Int32 => {
                let raster = band.read_band_as::<i32>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::UInt64 => {
                let raster = band.read_band_as::<u64>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::Int64 => {
                let raster = band.read_band_as::<i64>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::Float32 => {
                let raster = band.read_band_as::<f32>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            GdalDataType::Float64 => {
                let raster = band.read_band_as::<f64>()?;
                compute_entries_from_data(
                    raster.data(),
                    total_pixels,
                    width,
                    gt,
                    grid.as_ref(),
                    refinement_level,
                )
            }
            _ => Err(EncodingError::GeoTiff(format!(
                "unsupported GDAL data type: {band_type:?}"
            ))),
        }?;

        let linear_values: BTreeMap<u64, Vec<u8>> = computed_entries.into_iter().collect();

        backend.create_level(refinement_level.get() as u32, band_index as u32, zones)?;

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
            backend.write_chunk(
                refinement_level.get() as u32,
                band_index as u32,
                chunk_index,
                &bytes,
            )?;
        }
    }

    Ok(backend)
}
