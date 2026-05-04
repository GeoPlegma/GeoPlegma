use std::path::Path;

use gdal::raster::GdalDataType;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::{Dataset, GeoTransformEx};
use indicatif::{ProgressBar, ProgressStyle};
use geoplegma::api::DggrsApiConfig;
use geoplegma::get;
use geoplegma::types::{BoundingBox, DggrsUid, Point, RefinementLevel, RelativeDepth};

use crate::AttributeSchema;
use crate::common::CONFIG;
use crate::error::EncodingError;
use crate::models::{Compression, DataType, DatasetMetadata};
use crate::stats::{BandStatsCollector, ConversionReport};
use crate::storage::StorageBackend;

trait NativeBytes {
    fn to_native_bytes(self) -> Vec<u8>;
    fn to_f64(self) -> f64;
}

macro_rules! impl_native_bytes {
    ($($t:ty),+ $(,)?) => {
        $(
            impl NativeBytes for $t {
                fn to_native_bytes(self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }
                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )+
    };
}

impl_native_bytes!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);

const ZARR_TARGET_UNCOMPRESSED_CHUNK_BYTES: u64 = 1024 * 1024;

fn get_corners_and_pixel_size(
    dataset: &Dataset,
) -> Result<(Option<BoundingBox>, f64, f64), EncodingError> {
    let (width_px, height_px) = dataset.raster_size();
    let w = width_px as f64;
    let h = height_px as f64;

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref()?;

    let mut wgs84 = SpatialRef::from_epsg(4326)?;
    wgs84.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);
    let to_wgs84 = CoordTransform::new(&src_srs, &wgs84)?;

    let (ulx, uly) = gt.apply(0.0, 0.0);
    let (urx, ury) = gt.apply(w, 0.0);
    let (lrx, lry) = gt.apply(w, h);
    let (llx, lly) = gt.apply(0.0, h);

    let mut xs = vec![ulx, urx, lrx, llx];
    let mut ys = vec![uly, ury, lry, lly];
    let mut zs = vec![];
    to_wgs84.transform_coords(&mut xs, &mut ys, &mut zs)?;

    let lon_min = xs.iter().fold(f64::INFINITY, |acc, x| acc.min(*x));
    let lon_max = xs.iter().fold(f64::NEG_INFINITY, |acc, x| acc.max(*x));
    let lat_min = ys.iter().fold(f64::INFINITY, |acc, y| acc.min(*y));
    let lat_max = ys.iter().fold(f64::NEG_INFINITY, |acc, y| acc.max(*y));

    println!("Bounding box (WGS84):");
    println!("  lon: [{lon_min:.6}, {lon_max:.6}]");
    println!("  lat: [{lat_min:.6}, {lat_max:.6}]");

    let cx = gt[0] + (w / 2.0) * gt[1] + (h / 2.0) * gt[2];
    let cy = gt[3] + (w / 2.0) * gt[4] + (h / 2.0) * gt[5];

    let metric = SpatialRef::from_epsg(3857)?;
    let to_metric = CoordTransform::new(&src_srs, &metric)?;

    let mut px = vec![cx, cx + gt[1], cx + gt[2]];
    let mut py = vec![cy, cy + gt[4], cy + gt[5]];
    let mut pz = vec![];
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
        Some(BoundingBox::new(lon_min, lat_min, lon_max, lat_max))
    };

    Ok((bbox, pixel_w, pixel_h))
}

fn nearest_pixel_index_for_center(
    center: &Point,
    wgs84_to_src: &CoordTransform,
    gt: [f64; 6],
    width: usize,
    height: usize,
) -> Result<Option<usize>, EncodingError> {
    if width == 0 || height == 0 {
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let mut xs = vec![center.lon];
    let mut ys = vec![center.lat];
    let mut zs = vec![];
    wgs84_to_src.transform_coords(&mut xs, &mut ys, &mut zs)?;

    let det = gt[1] * gt[5] - gt[2] * gt[4];
    if det.abs() < f64::EPSILON {
        return Err(EncodingError::GeoTiff(
            "geotransform is not invertible".into(),
        ));
    }

    let dx = xs[0] - gt[0];
    let dy = ys[0] - gt[3];

    // Inverse affine transform gives corner-based pixel coordinates.
    // Subtract 0.5 so rounding selects the nearest pixel center.
    let col_corner = (gt[5] * dx - gt[2] * dy) / det;
    let row_corner = (-gt[4] * dx + gt[1] * dy) / det;

    let col = (col_corner - 0.5).round();
    let row = (row_corner - 0.5).round();

    // Do not clamp. If center is outside raster extent, keep fill value.
    if !(0.0..(width as f64)).contains(&col) || !(0.0..(height as f64)).contains(&row) {
        return Ok(None);
    }

    Ok(Some((row as usize) * width + (col as usize)))
}

fn compute_chunk_bytes_from_data<T>(
    data: &[T],
    width: usize,
    height: usize,
    gt: [f64; 6],
    wgs84_to_src: &CoordTransform,
    chunk_child_centers: &[Vec<Point>],
    chunk_size: u64,
    stats: &mut BandStatsCollector,
) -> Result<Vec<Vec<u8>>, EncodingError>
where
    T: NativeBytes + Copy,
{
    let expected_pixels = width
        .checked_mul(height)
        .ok_or_else(|| EncodingError::GeoTiff("raster pixel count overflow".into()))?;
    if data.len() != expected_pixels {
        return Err(EncodingError::GeoTiff(format!(
            "raster data length {} does not match expected pixel count {}",
            data.len(),
            expected_pixels
        )));
    }

    let value_size = std::mem::size_of::<T>();
    let mut all_chunks = Vec::with_capacity(chunk_child_centers.len());

    for child_centers in chunk_child_centers {
        let mut bytes = vec![0_u8; chunk_size as usize * value_size];

        for (in_chunk_index, center) in child_centers.iter().enumerate() {
            if let Some(pixel_index) =
                nearest_pixel_index_for_center(center, wgs84_to_src, gt, width, height)?
            {
                let value = data[pixel_index];
                stats.record_value(value.to_f64());
                let value_bytes = value.to_native_bytes();

                let start = in_chunk_index * value_size;
                let end = start + value_size;
                bytes[start..end].copy_from_slice(&value_bytes);
            }
        }

        all_chunks.push(bytes);
    }

    Ok(all_chunks)
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

fn choose_best_chunk_level_and_size(
    refinement_level: RefinementLevel,
    min_chunk_level: RefinementLevel,
    max_relative_depth_allowed: RelativeDepth,
    aperture: u64,
    data_type_size_bytes: usize,
) -> Result<(RefinementLevel, u64), EncodingError> {
    if data_type_size_bytes == 0 {
        return Err(EncodingError::Storage(
            "data type size must be greater than zero".into(),
        ));
    }
    if aperture < 2 {
        return Err(EncodingError::Grid(
            "grid aperture must be at least 2 for chunk sizing".into(),
        ));
    }
    if min_chunk_level.get() > refinement_level.get() {
        return Err(EncodingError::Storage(format!(
            "min chunk level ({}) is greater than refinement level ({})",
            min_chunk_level.get(),
            refinement_level.get()
        )));
    }

    let max_relative_depth_from_levels = refinement_level.get() - min_chunk_level.get();
    let max_relative_depth = max_relative_depth_from_levels.min(max_relative_depth_allowed.get());
    let target_bytes = ZARR_TARGET_UNCOMPRESSED_CHUNK_BYTES as u128;
    let dtype_bytes = data_type_size_bytes as u128;

    let mut best_relative_depth = 0i32;
    let mut best_chunk_cells = 1u128;
    let mut best_diff = target_bytes.abs_diff(dtype_bytes); // depth 0 => 1 cell

    let mut chunk_cells = 1u128;
    for relative_depth in 1..=max_relative_depth {
        chunk_cells = chunk_cells.checked_mul(aperture as u128).ok_or_else(|| {
            EncodingError::Storage("chunk size overflow while computing aperture growth".into())
        })?;

        let chunk_bytes = chunk_cells.checked_mul(dtype_bytes).ok_or_else(|| {
            EncodingError::Storage("chunk byte size overflow while tuning chunk level".into())
        })?;

        let diff = target_bytes.abs_diff(chunk_bytes);
        if diff < best_diff {
            best_diff = diff;
            best_relative_depth = relative_depth;
            best_chunk_cells = chunk_cells;
        }
    }

    let chunk_level = RefinementLevel::new_const(refinement_level.get() - best_relative_depth);
    let chunk_size = u64::try_from(best_chunk_cells)
        .map_err(|_| EncodingError::Storage("chunk size does not fit into u64".into()))?;

    let chunk_size = chunk_size * 1.05 as u64;

    Ok((chunk_level, chunk_size))
}

pub fn convert_geotiff_file_to_backend<B>(
    geotiff_path: &Path,
    output_path: &Path,
    dggrs: DggrsUid,
    compression: Option<Compression>,
) -> Result<(B, ConversionReport), EncodingError>
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

    let (width, height) = dataset.raster_size();

    if width == 0 || height == 0 {
        return Err(EncodingError::GeoTiff(
            "raster has zero width or height".into(),
        ));
    }

    let gt = dataset.geo_transform()?;
    let src_srs = dataset.spatial_ref()?;
    let (bbox, pixel_width, pixel_height) = get_corners_and_pixel_size(&dataset)?;
    let refinement_level = get_closest_refinement_level(&grid, pixel_width, pixel_height)?;
    // let refinement_level = RefinementLevel::new_const(get_closest_refinement_level(&grid, pixel_width, pixel_height)?.get() - 3);

    let data_type_size_bytes = metadata_bands
        .iter()
        .map(|band| band.dtype.byte_size())
        .max()
        .ok_or_else(|| {
            EncodingError::Storage("dataset metadata must define at least one attribute".into())
        })?;
    let min_chunk_level = grid.min_refinement_level()?;
    let max_relative_depth_allowed = grid.max_relative_depth()?;
    let (chunk_level, chunk_size) = choose_best_chunk_level_and_size(
        refinement_level,
        min_chunk_level,
        max_relative_depth_allowed,
        dggrs.spec().aperture as u64,
        data_type_size_bytes,
    )?;
    println!(
        "refinement level: {}, chunk level: {}, chunk size: {}, chunk bytes: {}",
        refinement_level.get(),
        chunk_level.get(),
        chunk_size,
        chunk_size * data_type_size_bytes as u64
    );

    let chunk_zones = grid.zones_from_bbox(chunk_level, bbox, Some(CONFIG))?;
    if chunk_zones.zones.is_empty() {
        return Err(EncodingError::Grid(
            "no zones found intersecting dataset bounding box".into(),
        ));
    }
    println!("zones in bbox: {}", chunk_zones.zones.len());

    let relative_depth = RelativeDepth::new(refinement_level.get() - chunk_level.get())?;
    let center_config = DggrsApiConfig {
        center: true,
        ..CONFIG
    };
    let total_chunk_zones = chunk_zones.zones.len();
    let chunk_progress = ProgressBar::new(total_chunk_zones as u64);
    let style = ProgressStyle::with_template(
        "processing chunk zones [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
    )
    .map_err(|e| EncodingError::Storage(format!("invalid progress bar template: {e}")))?
    .progress_chars("=> ");
    chunk_progress.set_style(style);

    let chunk_child_centers: Vec<Vec<Point>> = chunk_zones
        .zones
        .iter()
        .map(|chunk_zone| {
            let children =
                grid.zones_from_parent(relative_depth, chunk_zone.id.clone(), Some(center_config))?;
            if children.zones.len() > chunk_size as usize {
                return Err(EncodingError::Grid(format!(
                    "chunk {} has {} children but chunk_size is {}",
                    chunk_zone.id,
                    children.zones.len(),
                    chunk_size
                )));
            }

            children
                .zones
                .iter()
                .map(|child| {
                    child.center.ok_or_else(|| {
                        EncodingError::Grid(format!(
                            "zone {} in chunk {} has no center coordinates",
                            child.id, chunk_zone.id
                        ))
                    })
                })
                .collect::<Result<Vec<_>, EncodingError>>()
                .inspect(|_| chunk_progress.inc(1))
        })
        .collect::<Result<_, EncodingError>>()?;
    chunk_progress.finish_with_message("processing chunk zones [done]");

    let band_dtype_names: Vec<String> = metadata_bands
        .iter()
        .map(|b| format!("{:?}", b.dtype))
        .collect();

    let metadata = DatasetMetadata {
        dggrs,
        attributes: metadata_bands,
        chunk_size,
        chunk_ids: chunk_zones
            .zones
            .iter()
            .map(|z| z.id.to_string())
            .collect::<Vec<_>>(),
        levels: vec![refinement_level.get() as u32],
        compression,
    };

    let encoded_num_cells = (chunk_zones.zones.len() as u64)
        .checked_mul(chunk_size)
        .ok_or_else(|| EncodingError::Storage("encoded cell count overflow".into()))?;
    let mut backend = B::create(output_path, metadata)?;

    let mut wgs84 = SpatialRef::from_epsg(4326)?;
    wgs84.set_axis_mapping_strategy(gdal::spatial_ref::AxisMappingStrategy::TraditionalGisOrder);
    let wgs84_to_src = CoordTransform::new(&wgs84, &src_srs)?;

    let mut band_stats = Vec::new();

    for (band_index, band_type) in bands.into_iter().enumerate() {
        let band = dataset.rasterband(band_index + 1)?;
        let dtype_name = band_dtype_names[band_index].clone();
        let mut collector = BandStatsCollector::new(band_index as u32, dtype_name);
        collector.set_total_cells(encoded_num_cells);

        let chunk_bytes = match band_type {
            GdalDataType::UInt8 => {
                let raster = band.read_band_as::<u8>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::Int8 => {
                let raster = band.read_band_as::<i8>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::UInt16 => {
                let raster = band.read_band_as::<u16>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::Int16 => {
                let raster = band.read_band_as::<i16>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::UInt32 => {
                let raster = band.read_band_as::<u32>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::Int32 => {
                let raster = band.read_band_as::<i32>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::UInt64 => {
                let raster = band.read_band_as::<u64>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::Int64 => {
                let raster = band.read_band_as::<i64>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::Float32 => {
                let raster = band.read_band_as::<f32>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            GdalDataType::Float64 => {
                let raster = band.read_band_as::<f64>()?;
                compute_chunk_bytes_from_data(
                    raster.data(),
                    width,
                    height,
                    gt,
                    &wgs84_to_src,
                    &chunk_child_centers,
                    chunk_size,
                    &mut collector,
                )
            }
            _ => Err(EncodingError::GeoTiff(format!(
                "unsupported GDAL data type: {band_type:?}"
            ))),
        }?;

        band_stats.push(collector.finish());

        backend.create_level(
            refinement_level.get() as u32,
            band_index as u32,
            encoded_num_cells,
        )?;

        for (chunk_index, bytes) in chunk_bytes.iter().enumerate() {
            backend.write_chunk(
                refinement_level.get() as u32,
                band_index as u32,
                chunk_index as u64,
                bytes,
            )?;
        }
    }

    let report = ConversionReport {
        num_chunks: chunk_zones.zones.len() as u64,
        chunk_size,
        refinement_level: refinement_level.get() as u32,
        bands: band_stats,
    };

    Ok((backend, report))
}
