use std::collections::BTreeMap;
use std::path::Path;

use gdal::raster::GdalDataType;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::{Dataset, GeoTransformEx};
use geo::Rect;
use geo_types::Point;
use geoplegma::api::DggrsApi;
use geoplegma::get;
use geoplegma::models::common::{DggrsUid, RefinementLevel};
use rayon::prelude::*;

use crate::AttributeSchema;
use crate::common::{CONFIG, zone_id_to_u64};
use crate::error::EncodingError;
use crate::models::{DataType, DatasetMetadata};
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

fn get_corners_and_pixel_size(
    dataset: &Dataset,
) -> Result<(Option<Rect<f64>>, f64, f64), EncodingError> {
    let (width_px, height_px) = dataset.raster_size();
    let w = width_px as f64;
    let h = height_px as f64;

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref()?;

    let mut wgs84 = SpatialRef::from_epsg(4326)?;
    wgs84.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);
    let to_wgs84 = CoordTransform::new(&src_srs, &wgs84)?;

    let mut xs = vec![gt[0], gt[0] + w * gt[1] + h * gt[2]];
    let mut ys = vec![gt[3], gt[3] + w * gt[4] + h * gt[5]];
    let mut zs = vec![0.0f64; 2];
    to_wgs84.transform_coords(&mut xs, &mut ys, &mut zs)?;

    let (lon_min, lon_max) = (xs[0].min(xs[1]), xs[0].max(xs[1]));
    let (lat_min, lat_max) = (ys[0].min(ys[1]), ys[0].max(ys[1]));

    println!("Bounding box (WGS84):");
    println!("  lon: [{lon_min:.6}, {lon_max:.6}]");
    println!("  lat: [{lat_min:.6}, {lat_max:.6}]");

    let cx = gt[0] + (w / 2.0) * gt[1] + (h / 2.0) * gt[2];
    let cy = gt[3] + (w / 2.0) * gt[4] + (h / 2.0) * gt[5];

    let metric = SpatialRef::from_epsg(3857)?;
    let to_metric = CoordTransform::new(&src_srs, &metric)?;

    let mut px = vec![cx, cx + gt[1], cx + gt[2]];
    let mut py = vec![cy, cy + gt[4], cy + gt[5]];
    let mut pz = vec![0.0f64; 3];
    to_metric.transform_coords(&mut px, &mut py, &mut pz)?;

    let pixel_w = f64::hypot(px[1] - px[0], py[1] - py[0]);
    let pixel_h = f64::hypot(px[2] - px[0], py[2] - py[0]);

    let tolerance = 1e-4; // 0.0001 degrees
    let is_global = (lon_min <= -180.0 + tolerance)
        && (lon_max >= 180.0 - tolerance)
        && (lat_min <= -90.0 + tolerance)
        && (lat_max >= 90.0 - tolerance);

    let bbox = if is_global {
        None
    } else {
        Some(Rect::new(
            Point::new(lon_min, lat_min),
            Point::new(lon_max, lat_max),
        ))
    };

    Ok((bbox, pixel_w, pixel_h))
}

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
    pixel_width: f64,
    pixel_height: f64,
) -> Result<RefinementLevel, EncodingError> {
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
    let diff_percentage = (best_diff as f64 / world_pixel_count as f64) * 100.0;
    println!(
        "best level: {} with diff {}",
        best_level.unwrap().get(),
        diff_percentage
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
    let (bbox, pixel_width, pixel_height) = get_corners_and_pixel_size(&dataset)?;
    let refinement_level = get_closest_refinement_level(&grid, pixel_width, pixel_height)?;

    let chunk_level = RefinementLevel::new_const(
        (refinement_level.get() - 4).max(grid.min_refinement_level()?.get()),
    );
    let chunk_size = dggrs
        .spec()
        .aperture
        .pow((refinement_level.get() - chunk_level.get()) as u32) as u64;
    println!(
        "refinement level: {}, chunk level: {}, chunk size: {}",
        refinement_level.get(),
        chunk_level.get(),
        chunk_size
    );

    let chunk_zones = grid.zones_from_bbox(chunk_level, bbox, Some(CONFIG))?;
    if chunk_zones.zones.is_empty() {
        return Err(EncodingError::Grid(
            "no zones found intersecting dataset bounding box".into(),
        ));
    }
    println!("zones in bbox: {}", chunk_zones.zones.len());

    let metadata = DatasetMetadata {
        dggrs,
        attributes: metadata_bands,
        chunk_size,
        chunk_ids: chunk_zones
            .zones
            .iter()
            .map(|z| zone_id_to_u64(&z.id))
            .collect::<Result<_, _>>()?,
        levels: vec![refinement_level.get() as u32],
        compression: None,
    };

    let zones = grid.zone_count(refinement_level)?;
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
