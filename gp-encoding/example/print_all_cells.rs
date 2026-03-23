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
    println!("Chunk Size:  {}", metadata.chunk_size);
    println!("Attributes:  {}", metadata.attributes.len());
    println!();

    let attributes = &metadata.attributes;
    if attributes.is_empty() {
        eprintln!("No attributes defined in metadata");
        return;
    }

    let cell_stride: usize = attributes.iter().map(|a| a.dtype.byte_size()).sum();
    if cell_stride == 0 {
        eprintln!("Invalid metadata: computed cell stride is 0");
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
                            if chunk_data.len() % cell_stride != 0 {
                                eprintln!(
                                    "  Warning: chunk {} byte length {} is not a multiple of cell stride {}",
                                    chunk_idx,
                                    chunk_data.len(),
                                    cell_stride
                                );
                            }
                            let num_values = chunk_data.len() / cell_stride;

                            for cell_offset in 0..num_values {
                                let cell_index = chunk_idx * metadata.chunk_size as u64 + cell_offset as u64;
                                let start = cell_offset * cell_stride;
                                let end = start + cell_stride;

                                if end <= chunk_data.len() {
                                    let value_bytes = &chunk_data[start..end];
                                    let mut offset = 0usize;
                                    let mut values = Vec::with_capacity(attributes.len());

                                    for (band_idx, attribute) in attributes.iter().enumerate() {
                                        let band_size = attribute.dtype.byte_size();
                                        let band_end = offset + band_size;
                                        if band_end <= value_bytes.len() {
                                            let band_bytes = &value_bytes[offset..band_end];
                                            let value_str = format_value(&attribute.dtype, band_bytes);
                                            values.push(format!("band{}={}", band_idx + 1, value_str));
                                        } else {
                                            values.push(format!("band{}=INVALID", band_idx + 1));
                                        }
                                        offset = band_end;
                                    }

                                    println!("  Cell {}: {}", cell_index, values.join(", "));
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
