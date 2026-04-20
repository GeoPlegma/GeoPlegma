use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use geoplegma::types::{DggrsUid, Point, RefinementLevel};
use gp_encoding::{
    StorageBackend, ZarrBackend, convert_geotiff_file_to_backend,
    query_value_for_point,
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

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::ConvertGeotiff(args) => run_convert_geotiff(args),
        Commands::Query(args) => run_query(args),
        Commands::Stats(args) => run_stats(args),
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

    let backend =
        convert_geotiff_file_to_backend::<ZarrBackend>(&args.input, &args.output, args.dggrs)
            .map_err(|e| e.to_string())?;

    println!("Conversion successful");
    println!("  Input:      {}", args.input.display());
    println!("  Output:     {}", args.output.display());
    println!("  Levels:     {:?}", backend.levels());

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

fn format_value(dtype: &gp_encoding::DataType, bytes: &[u8]) -> Result<String, String> {
    use gp_encoding::DataType;

    match dtype {
        DataType::Float32 => parse_fixed::<4>(bytes).map(|arr| f32::from_ne_bytes(arr).to_string()),
        DataType::Float64 => parse_fixed::<8>(bytes).map(|arr| f64::from_ne_bytes(arr).to_string()),
        DataType::Int8 => parse_fixed::<1>(bytes).map(|arr| i8::from_ne_bytes(arr).to_string()),
        DataType::Int16 => parse_fixed::<2>(bytes).map(|arr| i16::from_ne_bytes(arr).to_string()),
        DataType::Int32 => parse_fixed::<4>(bytes).map(|arr| i32::from_ne_bytes(arr).to_string()),
        DataType::Int64 => parse_fixed::<8>(bytes).map(|arr| i64::from_ne_bytes(arr).to_string()),
        DataType::UInt8 => parse_fixed::<1>(bytes).map(|arr| u8::from_ne_bytes(arr).to_string()),
        DataType::UInt16 => parse_fixed::<2>(bytes).map(|arr| u16::from_ne_bytes(arr).to_string()),
        DataType::UInt32 => parse_fixed::<4>(bytes).map(|arr| u32::from_ne_bytes(arr).to_string()),
        DataType::UInt64 => parse_fixed::<8>(bytes).map(|arr| u64::from_ne_bytes(arr).to_string()),
    }
}

fn parse_fixed<const N: usize>(bytes: &[u8]) -> Result<[u8; N], String> {
    if bytes.len() < N {
        return Err(format!(
            "not enough bytes to decode value: expected at least {N}, got {}",
            bytes.len()
        ));
    }

    let mut arr = [0_u8; N];
    arr.copy_from_slice(&bytes[..N]);
    Ok(arr)
}
