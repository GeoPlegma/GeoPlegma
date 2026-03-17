//! Example: GeoTIFF -> Encoded Storage
//!
//! Usage:
//! ```sh
//! cargo run -p gp-encoding --example geotiff_convert -- <path/to/input.tif>
//! ```

use std::path::PathBuf;

use geoplegma::adapters::h3o::h3::H3Impl;
use geoplegma::models::common::{DggrsUid, RefinementLevel};
use gp_encoding::{
    AttributeSchema, DataType, DatasetMetadata, GridExtent, StorageBackend, ZarrBackend,
    convert_geotiff_file_to_backend,
};

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p gp-encoding --example geotiff_convert -- <input.tif>");
        std::process::exit(2);
    });
    let refinement = std::env::args()
        .nth(2)
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(5);

    let input_path = PathBuf::from(input);
    let output_store = PathBuf::from("./tmp/gp_encoding_geotiff_convert");

    if output_store.exists() {
        std::fs::remove_dir_all(&output_store).expect("clean up output");
    }

    let metadata = DatasetMetadata {
        dggrs: DggrsUid::H3,
        extent: GridExtent::Global,
        attributes: vec![AttributeSchema {
            name: "value_0".to_string(),
            dtype: DataType::Float32,
            fill_value: Some("0.0".to_string()),
        }],
        chunk_size: 1024,
        levels: vec![u32::from(refinement)],
        compression: None,
    };

    let backend = convert_geotiff_file_to_backend::<ZarrBackend, _>(
        &input_path,
        &output_store,
        &H3Impl::default(),
        RefinementLevel::from(refinement),
        0,
        metadata,
    )
    .expect("convert GeoTIFF to encoded dataset");

    println!("Conversion successful");
    println!("  Input:  {}", input_path.display());
    println!("  Output: {}", output_store.display());
    println!("  Refinement: {refinement}");
    println!("  Levels: {:?}", backend.levels());
}
