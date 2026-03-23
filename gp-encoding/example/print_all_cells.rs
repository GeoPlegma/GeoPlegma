//! Example: Print Every Cell in Zarr Store
//!
//! Iterates through all resolution levels and cells in a Zarr store,
//! printing each cell's index and value.
//!
//! First run the zarr_roundtrip example to create a store, then:
//!
//! ```sh
//! cargo run -p gp-encoding --example print_all_cells
//! ```

use gp_encoding::{ZarrBackend, StorageBackend};
use std::path::Path;

fn main() {
    let store_path = Path::new("./tmp/gp_encoding_geotiff_convert");

    if !store_path.exists() {
        eprintln!("Zarr store not found at {}. Please run the zarr_roundtrip example first.", store_path.display());
        std::process::exit(1);
    }

    let backend = ZarrBackend::open(store_path).expect("Failed to open Zarr store");

    let metadata = backend.metadata();
    println!("=== Zarr Store Contents ===");
    println!("DGGRS:       {}", metadata.dggrs);
    println!("Attributes:  {:?}", metadata.attributes.iter().map(|a| &a.name).collect::<Vec<_>>());
    println!("Chunk Size:  {}", metadata.chunk_size);
    println!();

    let attributes = &metadata.attributes;
    if attributes.is_empty() {
        eprintln!("No attributes defined in metadata");
        return;
    }

    // Iterate through all resolution levels
    let levels = backend.levels();
    for level in levels {
        println!("─── Level {} ───", level);

        match backend.num_chunks(level) {
            Ok(num_chunks) => {
                let mut total_cells = 0;
                for chunk_idx in 0..num_chunks {
                    match backend.read_chunk(level, chunk_idx) {
                        Ok(chunk_data) => {
                            let value_size: usize = attributes[0].dtype.byte_size();
                            let num_values = chunk_data.len() / value_size;

                            for cell_offset in 0..num_values {
                                let cell_index = chunk_idx * metadata.chunk_size as u64 + cell_offset as u64;
                                let start = cell_offset * value_size;
                                let end = start + value_size;

                                if end <= chunk_data.len() {
                                    let value_bytes = &chunk_data[start..end];
                                    let value_str = format_value(&attributes[0].dtype, value_bytes);
                                    println!("  Cell {}: {} {}", cell_index, attributes[0].name, value_str);
                                    total_cells += 1;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("  Error reading chunk {}: {}", chunk_idx, e);
                        }
                    }
                }
                println!("  Total cells: {}\n", total_cells);
            }
            Err(e) => {
                eprintln!("  Error getting chunk count: {}\n", e);
            }
        }
    }
}

/// Parse and format a value based on its data type
fn format_value(dtype: &gp_encoding::DataType, bytes: &[u8]) -> String {
    use gp_encoding::DataType;

    match dtype {
        DataType::Float32 => {
            if bytes.len() >= 4 {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&bytes[..4]);
                format!("{}", f32::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::Float64 => {
            if bytes.len() >= 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8]);
                format!("{}", f64::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::Int8 => {
            if bytes.len() >= 1 {
                format!("{}", bytes[0] as i8)
            } else {
                "INVALID".to_string()
            }
        }
        DataType::Int16 => {
            if bytes.len() >= 2 {
                let mut arr = [0u8; 2];
                arr.copy_from_slice(&bytes[..2]);
                format!("{}", i16::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::Int32 => {
            if bytes.len() >= 4 {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&bytes[..4]);
                format!("{}", i32::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::Int64 => {
            if bytes.len() >= 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8]);
                format!("{}", i64::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::UInt8 => {
            if bytes.len() >= 1 {
                format!("{}", bytes[0])
            } else {
                "INVALID".to_string()
            }
        }
        DataType::UInt16 => {
            if bytes.len() >= 2 {
                let mut arr = [0u8; 2];
                arr.copy_from_slice(&bytes[..2]);
                format!("{}", u16::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::UInt32 => {
            if bytes.len() >= 4 {
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&bytes[..4]);
                format!("{}", u32::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
        DataType::UInt64 => {
            if bytes.len() >= 8 {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&bytes[..8]);
                format!("{}", u64::from_le_bytes(arr))
            } else {
                "INVALID".to_string()
            }
        }
    }
}
