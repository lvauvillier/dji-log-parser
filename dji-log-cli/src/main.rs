use clap::Parser;
use dji_log_parser::record::Record;
use dji_log_parser::{DJILog, DecryptMethod};
use exporters::{GeoJsonExporter, ImageExporter, JsonExporter, KmlExporter};
use std::fs;

mod exporters;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Input log file
    #[arg(value_name = "FILE")]
    filepath: String,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,

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
    #[arg(short, long)]
    kml: Option<String>,

    /// DJI keychain Api Key
    #[arg(short, long)]
    api_key: Option<String>,
}

pub(crate) trait Exporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, args: &Cli);
}

fn main() {
    let args = Cli::parse();

    let bytes = fs::read(&args.filepath).expect("Unable to read file");
    let parser = DJILog::from_bytes(&bytes).expect("Unable to parse file");

    // Configure a decrypt method
    let decrypt_method = if parser.version >= 13 {
        if let Some(api_key) = &args.api_key {
            DecryptMethod::ApiKey(api_key.clone())
        } else {
            panic!("Api Key required");
        }
    } else {
        DecryptMethod::None
    };

    let records: Vec<Record> = parser
        .records(decrypt_method)
        .expect("Unable to parse records");

    let exporters: Vec<&dyn Exporter> = vec![
        &JsonExporter,
        &ImageExporter,
        &GeoJsonExporter,
        &KmlExporter,
    ];

    for exporter in exporters {
        exporter.export(&parser, &records, &args);
    }
}
