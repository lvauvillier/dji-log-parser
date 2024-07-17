use clap::Parser;
use dji_log_parser::frame::Frame;
use dji_log_parser::record::Record;
use dji_log_parser::{DJILog, DecryptMethod};
use exporters::{CSVExporter, GeoJsonExporter, ImageExporter, JsonExporter, KmlExporter};
use std::fs::{self, File};

// temp
use std::io::Write;
use log::{debug, error, info};
use env_logger;

mod exporters;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Input log file
    #[arg(value_name = "FILE")]
    filepath: String,

    /// Output file path
    #[arg(short, long, conflicts_with = "keychains_output", conflicts_with = "kml")]
    output: Option<String>,

    /// Output file path for keychains
    #[arg(short = 'k', long = "keychains-output")]
    keychains_output: Option<String>,

    /// Image file path.
    #[arg(short, long)]
    images: Option<String>,

    /// Thumbnail file path.
    #[arg(short, long)]
    thumbnails: Option<String>,

    /// GeoJSON file path.
    #[arg(short, long)]
    geojson: Option<String>,

    /// KML file path.
    #[arg(short = 'm', long)]  // Changed short option to -m
    kml: Option<String>,

    /// CSV file path.
    #[arg(short = 'c', long)]  // Changed short option to -c
    csv: Option<String>,

    /// DJI keychain Api Key
    #[arg(short, long)]
    api_key: Option<String>,
}

pub(crate) trait Exporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let args = Cli::parse();
    debug!("CLI arguments parsed: {:?}", args);

    let bytes = fs::read(&args.filepath).map_err(|e| {
        error!("Failed to read file: {}", e);
        e
    })?;
    info!("File read successfully, size: {} bytes", bytes.len());

    let parser = DJILog::from_bytes(bytes).map_err(|e| {
        error!("Failed to parse DJI log: {}", e);
        e
    })?;
    info!("DJI log parsed successfully, version: {}", parser.version);

    // Configure a decrypt method
    let decrypt_method = if parser.version >= 13 {
        if let Some(api_key) = &args.api_key {
            debug!("Fetching keychains with API key");
            let keychains = parser.keychain_request().map_err(|e| {
                error!("Failed to create keychain request: {}", e);
                e
            })?.fetch(api_key).map_err(|e| {
                error!("Failed to fetch keychains: {}", e);
                e
            })?;

            // Write fetched keychains to the output file if specified
            if let Some(keychains_output) = &args.keychains_output {
                let mut output_file = File::create(keychains_output).map_err(|e| {
                    error!("Failed to create keychains output file: {}", e);
                    e
                })?;
                writeln!(output_file, "Fetched keychains: {:?}", keychains).map_err(|e| {
                    error!("Failed to write keychains to file: {}", e);
                    e
                })?;
                info!("Keychains written to {}", keychains_output);
            }
            DecryptMethod::Keychains(keychains)
        } else {
            error!("API Key required for log version >= 13");
            return Err("API Key required".into());
        }
    } else {
        DecryptMethod::None
    };

    let records = parser.records(decrypt_method.clone()).map_err(|e| {
        error!("Failed to parse records: {}", e);
        e
    })?;
    info!("Records parsed successfully, count: {}", records.len());

    let frames = parser.frames(decrypt_method).map_err(|e| {
        error!("Failed to parse frames: {}", e);
        e
    })?;
    info!("Frames parsed successfully, count: {}", frames.len());

    let exporters: Vec<Box<dyn Exporter>> = vec![
        Box::new(JsonExporter),
        Box::new(ImageExporter),
        Box::new(GeoJsonExporter),
        Box::new(KmlExporter),
        Box::new(CSVExporter),
    ];

    for exporter in exporters {
        exporter.export(&parser, &records, &frames, &args);
    }

    Ok(())
}
