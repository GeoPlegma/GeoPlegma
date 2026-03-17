//! Example: Zarr Storage Round-Trip
//!
//! Demonstrates creating a DGGS dataset backed by Zarr, writing data at a
//! resolution level, reading it back, and verifying correctness.
//!
//! ```sh
//! cargo run -p gp-encoding --example zarr_roundtrip
//! ```

use std::path::Path;

use geoplegma::models::common::DggrsUid;
use gp_encoding::{
    AttributeSchema, DataType, DatasetMetadata, GridExtent, StorageBackend, ZarrBackend,
};

fn main() {
    let store_path = Path::new("./tmp/gp_encoding_zarr_example");

    // Clean up any previous run.
    if store_path.exists() {
        std::fs::remove_dir_all(store_path).expect("clean up");
    }

    let metadata = DatasetMetadata {
        dggrs: DggrsUid::H3,
        extent: GridExtent::BoundingBox {
            min_lon: -9.5,
            min_lat: 38.5,
            max_lon: -8.5,
            max_lat: 39.0,
        },
        attributes: vec![AttributeSchema {
            name: "elevation".into(),
            dtype: DataType::Float32,
            fill_value: Some("0.0".into()),
        }],
        chunk_size: 1024,
        levels: vec![0, 3, 5],
        compression: None,
    };

    // ── 2. Create the Zarr store ────────────────────────────────────
    let mut backend = ZarrBackend::create(store_path, metadata).expect("create store");

    println!("Created Zarr store at {}", store_path.display());
    println!("  DGGRS:       {}", backend.metadata().dggrs);
    println!("  Chunk size:  {}", backend.metadata().chunk_size);

    // ── 3. Create a resolution level ────────────────────────────────
    let num_cells: u64 = 2048;
    let level = 3;
    let _handle = backend
        .create_level(level, num_cells)
        .expect("create level");
    println!("\nCreated level {level} with {num_cells} cells");

    // ── 4. Write some chunks ────────────────────────────────────────
    // Generate synthetic elevation data (f32 → bytes).
    let chunk_size = backend.metadata().chunk_size as usize;
    let dtype_size = std::mem::size_of::<f32>();

    for chunk_idx in 0..backend.num_chunks(level).unwrap() {
        let remaining = (num_cells as usize).saturating_sub(chunk_idx as usize * chunk_size);
        let count = remaining.min(chunk_size);
        let mut buf = Vec::with_capacity(count * dtype_size);
        for i in 0..count {
            let val = (chunk_idx as f32) * 100.0 + (i as f32) * 0.1;
            buf.extend_from_slice(&val.to_ne_bytes());
        }
        backend
            .write_chunk(level, chunk_idx, &buf)
            .expect("write chunk");
    }

    println!("Wrote {} chunks", backend.num_chunks(level).unwrap());

    // ── 5. Re-open and read back ────────────────────────────────────
    let backend2 = ZarrBackend::open(store_path).expect("re-open store");

    assert_eq!(backend2.metadata().dggrs, DggrsUid::H3);
    println!("\nRe-opened store — metadata OK");

    let chunk_0 = backend2.read_chunk(level, 0).expect("read chunk 0");
    let first_f32 = f32::from_ne_bytes(chunk_0[..4].try_into().unwrap());
    println!("First value in chunk 0: {first_f32}");
    assert!((first_f32 - 0.0).abs() < 1e-6);

    let chunk_1 = backend2.read_chunk(level, 1).expect("read chunk 1");
    let first_f32_c1 = f32::from_ne_bytes(chunk_1[..4].try_into().unwrap());
    println!("First value in chunk 1: {first_f32_c1}");
    assert!((first_f32_c1 - 100.0).abs() < 1e-6);

    println!("\nRound-trip successful!");

    // Clean up.
    // std::fs::remove_dir_all(store_path).ok();
}
