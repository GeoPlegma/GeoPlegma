use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use geoplegma::types::{DggrsUid, Point, RefinementLevel};
use gp_encoding::{
    convert_geotiff_file_to_backend, format_value, query_value_for_point,
    write_h3_level_as_visualization_json, Compression, StorageBackend, ZarrBackend,
};

#[derive(Parser, Debug)]
#[command(
    name = "gp-encoding",
    about = "CLI for querying and encoding DGGS-backed datasets",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Convert a GeoTIFF file into a Zarr-backed encoded dataset.
    ConvertGeotiff(ConvertGeotiffArgs),
    /// Query a value at geographic coordinates.
    Query(QueryArgs),
    /// Print summary statistics for an encoded Zarr store.
    Stats(StatsArgs),
    /// Export an H3 store level as JSON for the visualization app.
    ExportH3Json(ExportH3JsonArgs),
}

#[derive(Args, Debug)]
struct ConvertGeotiffArgs {
    /// DGGRS to use for the output dataset.
    #[arg(short, long)]
    dggrs: DggrsUid,
    /// Input GeoTIFF path.
    #[arg(short, long)]
    input: PathBuf,
    /// Output Zarr store path.
    #[arg(short, long, default_value = "./tmp/gp_encoding_geotiff_convert")]
    output: PathBuf,
    /// Optional compression for Zarr chunks.
    #[arg(long, value_enum)]
    compression: Option<Compression>,
}

#[derive(Args, Debug)]
struct QueryArgs {
    /// Zarr store path.
    #[arg(short, long)]
    store: PathBuf,
    /// Refinement level.
    #[arg(short, long)]
    level: u8,
    /// Longitude in degrees.
    #[arg(long)]
    lon: f64,
    /// Latitude in degrees.
    #[arg(long)]
    lat: f64,
    /// Optional band index (0-based). If omitted, all bands are queried.
    #[arg(long)]
    band: Option<u32>,
}

#[derive(Args, Debug)]
struct StatsArgs {
    /// Zarr store path.
    #[arg(short, long)]
    store: PathBuf,
}

#[derive(Args, Debug)]
struct ExportH3JsonArgs {
    /// H3 Zarr store path.
    #[arg(short, long)]
    store: PathBuf,
    /// Refinement level to export.
    #[arg(short, long)]
    level: u32,
    /// Output JSON file path. If omitted, auto-detects visualization/public.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::ConvertGeotiff(args) => run_convert_geotiff(args),
        Commands::Query(args) => run_query(args),
        Commands::Stats(args) => run_stats(args),
        Commands::ExportH3Json(args) => run_export_h3_json(args),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run_convert_geotiff(args: ConvertGeotiffArgs) -> Result<(), String> {
    if args.output.exists() {
        std::fs::remove_dir_all(&args.output).map_err(|e| {
            format!(
                "failed to clean output store {}: {e}",
                args.output.display()
            )
        })?;
    }

    let (backend, report) = convert_geotiff_file_to_backend::<ZarrBackend>(
        &args.input,
        &args.output,
        args.dggrs,
        args.compression,
    )
    .map_err(|e| e.to_string())?;

    println!("Conversion successful");
    println!("  Input:      {}", args.input.display());
    println!("  Output:     {}", args.output.display());
    println!("  Levels:     {:?}", backend.levels());
    print!("{report}");

    Ok(())
}

fn run_query(args: QueryArgs) -> Result<(), String> {
    let backend = ZarrBackend::open(&args.store).map_err(|e| e.to_string())?;

    let refinement = RefinementLevel::from(args.level);
    let point = Point::new(args.lat, args.lon);

    let band_count = backend.band_count();
    let bands: Vec<u32> = if let Some(band) = args.band {
        if band >= band_count {
            return Err(format!(
                "band {band} is out of range (store has {band_count} bands)"
            ));
        }
        vec![band]
    } else {
        (0..band_count).collect()
    };

    println!(
        "Querying point ({}, {}) at level {}",
        args.lon, args.lat, args.level
    );

    for band in bands {
        let value_bytes = query_value_for_point(&backend, refinement, band, point)
            .map_err(|e| format!("query failed for band {band}: {e}"))?;
        let dtype = &backend.metadata().attributes[band as usize].dtype;
        let formatted = format_value(dtype, &value_bytes)
            .map_err(|e| format!("failed to decode value for band {band}: {e}"))?;
        println!("  band {band} ({dtype:?}): {formatted}");
    }

    Ok(())
}

fn run_stats(args: StatsArgs) -> Result<(), String> {
    let backend = ZarrBackend::open(&args.store).map_err(|e| e.to_string())?;
    let metadata = backend.metadata();

    println!("Store:       {}", args.store.display());
    println!("DGGRS:       {}", metadata.dggrs);
    println!("Chunk size:  {}", metadata.chunk_size);
    println!("Chunks:      {}", metadata.chunk_ids.len());
    println!("Bands:       {}", metadata.attributes.len());
    println!("Levels:      {:?}", backend.levels());

    let levels = backend.levels();
    let band_count = backend.band_count();

    for level in levels {
        let num_chunks = backend.num_chunks(level).map_err(|e| e.to_string())?;
        println!("\nLevel {level}");
        println!("  Logical chunk count: {num_chunks}");

        for band in 0..band_count {
            let dtype = &metadata.attributes[band as usize].dtype;

            println!("  Band {band} ({dtype:?})");
        }
    }

    Ok(())
}

fn run_export_h3_json(args: ExportH3JsonArgs) -> Result<(), String> {
    let backend = ZarrBackend::open(&args.store).map_err(|e| e.to_string())?;
    let output = resolve_visualization_output_path(args.output);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create output directory {}: {e}",
                parent.display()
            )
        })?;
    }

    let file = std::fs::File::create(&output)
        .map_err(|e| format!("failed to create JSON output {}: {e}", output.display()))?;
    let writer = std::io::BufWriter::new(file);
    let cell_count = write_h3_level_as_visualization_json(&backend, args.level, writer)
        .map_err(|e| e.to_string())?;

    println!("Export successful");
    println!("  Store:      {}", args.store.display());
    println!("  Level:      {}", args.level);
    println!("  Bands:      {}", backend.band_count());
    println!("  Cells:      {}", cell_count);
    println!("  Output:     {}", output.display());

    Ok(())
}

fn resolve_visualization_output_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }

    let local = PathBuf::from("./visualization/public/h3cells.json");
    if PathBuf::from("./visualization").is_dir() {
        return local;
    }

    let workspace = PathBuf::from("./gp-encoding/visualization/public/h3cells.json");
    if PathBuf::from("./gp-encoding/visualization").is_dir() {
        return workspace;
    }

    local
}
