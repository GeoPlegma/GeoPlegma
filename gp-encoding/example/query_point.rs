//! Example: Query a value at geographic coordinates
//!
//! Usage:
//! ```sh
//! cargo run -p gp-encoding --example query_point -- <path/to/zarr/store> <level> <lon> <lat>
//! ```
//!
//! Example:
//! ```sh
//! cargo run -p gp-encoding --example query_point -- ./tmp/gp_encoding_geotiff_convert 5 -8.5 38.7
//! ```

use std::path::PathBuf;

use geo_types::Point;
use geoplegma::models::common::RefinementLevel;
use gp_encoding::{query_value_bytes_for_point, StorageBackend, ZarrBackend};

fn main() {
    let store_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p gp-encoding --example query_point -- <store_path> <level> <lon> <lat>");
        std::process::exit(2);
    });

    let level = std::env::args()
        .nth(2)
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or_else(|| {
            eprintln!("usage: cargo run -p gp-encoding --example query_point -- <store_path> <level> <lon> <lat>");
            std::process::exit(2);
        });

    let lon = std::env::args()
        .nth(3)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| {
            eprintln!("usage: cargo run -p gp-encoding --example query_point -- <store_path> <level> <lon> <lat>");
            std::process::exit(2);
        });

    let lat = std::env::args()
        .nth(4)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or_else(|| {
            eprintln!("usage: cargo run -p gp-encoding --example query_point -- <store_path> <level> <lon> <lat>");
            std::process::exit(2);
        });

    let store_path = PathBuf::from(store_path);

    let backend = ZarrBackend::open(&store_path).expect("open zarr store");

    println!("Opened Zarr store at {}", store_path.display());
    println!("  DGGRS:       {}", backend.metadata().dggrs);
    println!("  Chunk size:  {}", backend.metadata().chunk_size);
    println!("  Levels:      {:?}", backend.levels());
    println!("  Attributes:  {:?}", 
        backend.metadata().attributes.iter()
            .map(|a| format!("{}", a.dtype as u8))
            .collect::<Vec<_>>()
    );

    let refinement = RefinementLevel::from(level);
    let point = Point::new(lon, lat);

    println!("\nQuerying point ({}, {}) at level {}", lon, lat, level);

    let value_bytes = query_value_bytes_for_point(&backend, refinement, point)
        .expect("query point");

    println!("Retrieved {} bytes", value_bytes.len());

    let dtype = &backend.metadata().attributes[0].dtype;
    match dtype {
        gp_encoding::DataType::Float32 => {
            let value = f32::from_ne_bytes(value_bytes[..4].try_into().unwrap());
            println!("Value (f32): {}", value);
        }
        gp_encoding::DataType::Float64 => {
            let value = f64::from_ne_bytes(value_bytes[..8].try_into().unwrap());
            println!("Value (f64): {}", value);
        }
        gp_encoding::DataType::Int8 => {
            let value = i8::from_ne_bytes(value_bytes[..1].try_into().unwrap());
            println!("Value (i8): {}", value);
        }
        gp_encoding::DataType::Int16 => {
            let value = i16::from_ne_bytes(value_bytes[..2].try_into().unwrap());
            println!("Value (i16): {}", value);
        }
        gp_encoding::DataType::Int32 => {
            let value = i32::from_ne_bytes(value_bytes[..4].try_into().unwrap());
            println!("Value (i32): {}", value);
        }
        gp_encoding::DataType::Int64 => {
            let value = i64::from_ne_bytes(value_bytes[..8].try_into().unwrap());
            println!("Value (i64): {}", value);
        }
        gp_encoding::DataType::UInt8 => {
            let value = u8::from_ne_bytes(value_bytes[..1].try_into().unwrap());
            println!("Value (u8): {}", value);
        }
        gp_encoding::DataType::UInt16 => {
            let value = u16::from_ne_bytes(value_bytes[..2].try_into().unwrap());
            println!("Value (u16): {}", value);
        }
        gp_encoding::DataType::UInt32 => {
            let value = u32::from_ne_bytes(value_bytes[..4].try_into().unwrap());
            println!("Value (u32): {}", value);
        }
        gp_encoding::DataType::UInt64 => {
            let value = u64::from_ne_bytes(value_bytes[..8].try_into().unwrap());
            println!("Value (u64): {}", value);
        }
    }
}
