use std::collections::BTreeMap;
use std::path::Path;

use gdal::raster::GdalDataType;
use gdal::{Dataset, GeoTransformEx};
use geo_types::Point;
use geoplegma::api::DggrsApi;
use geoplegma::get;
use geoplegma::models::common::RefinementLevel;
use rayon::prelude::*;

use crate::AttributeSchema;
use crate::common::zone_id_to_u64;
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

            let key = zone_id_to_u64(&zone.id)?;
            Ok((key, v.to_native_bytes()))
        })
        .collect()
}

pub fn convert_geotiff_file_to_backend<B>(
    geotiff_path: &Path,
    output_path: &Path,
    refinement_level: RefinementLevel,
    mut metadata: DatasetMetadata,
) -> Result<B, EncodingError>
where
    B: StorageBackend,
{
    let grid = get(metadata.dggrs).unwrap(); // TODO: remove unwrap

    let level = u32::try_from(refinement_level.get()).map_err(|_| {
        EncodingError::Grid(format!(
            "refinement level {} cannot be represented as u32",
            refinement_level.get()
        ))
    })?;

    let dataset = Dataset::open(geotiff_path).map_err(|e| EncodingError::Gdal(e.to_string()))?;

    let bands = dataset
        .rasterbands()
        .map(|b| b.unwrap().band_type())
        .collect::<Vec<_>>();
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

    metadata.attributes = metadata_bands;
    if metadata.attributes.is_empty() {
        return Err(EncodingError::Storage(
            "dataset metadata must define at least one attribute".into(),
        ));
    }

    let first_band = dataset
        .rasterband(1)
        .map_err(|e| EncodingError::Gdal(e.to_string()))?;

    let (width, height) = first_band.size();
    
    if width == 0 || height == 0 {
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let total_pixels = height * width;

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

    let zones = grid.zone_count(refinement_level).unwrap();
    let chunk_size = metadata.chunk_size;
    let mut backend = B::create(output_path, metadata)?;

    for (band_index, band_type) in bands.into_iter().enumerate() {
        let band = dataset
            .rasterband(band_index + 1)
            .map_err(|e| EncodingError::Gdal(e.to_string()))?;
        let value_size = band_type.bytes() as usize;

        let computed_entries = match band_type {
            GdalDataType::UInt8 => {
                let raster = band
                    .read_band_as::<u8>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<i8>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<u16>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<i16>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<u32>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<i32>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<u64>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<i64>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<f32>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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
                let raster = band
                    .read_band_as::<f64>()
                    .map_err(|e| EncodingError::Gdal(e.to_string()))?;
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


        backend.create_level(level, band_index as u32, zones)?;

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
            backend.write_chunk(level, band_index as u32, chunk_index, &bytes)?;
        }
    }

    Ok(backend)
}
