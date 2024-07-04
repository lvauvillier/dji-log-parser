use clap::Parser;
use dji_log_parser::frame::Frame;
use dji_log_parser::record::Record;
use dji_log_parser::{DJILog, DecryptMethod};
use exporters::{CSVExporter, GeoJsonExporter, ImageExporter, JsonExporter, KmlExporter};
use std::fs;

mod exporters;
mod utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    /// Input log file
    #[arg(value_name = "FILE")]
    filepath: String,

    /// Write JSON output to FILE instead of stdout
    #[arg(short, long)]
    output: Option<String>,

    /// Extract images (use %d for sequence, e.g., image%d.jpeg)
    #[arg(short, long)]
    images: Option<String>,

    /// Extract thumbnails (use %d for sequence, e.g., thumb%d.jpeg)
    #[arg(short, long)]
    thumbnails: Option<String>,

    /// Generate GeoJSON file
    #[arg(short, long)]
    geojson: Option<String>,

    /// Generate KML file
    #[arg(short, long)]
    kml: Option<String>,

    /// Generate CSV file
    #[arg(short, long)]
    csv: Option<String>,

    /// DJI keychain Api Key
    #[arg(short, long)]
    api_key: Option<String>,

    /// Extract raw records instead of normalized frames
    #[arg(short, long)]
    raw: bool,
}

pub(crate) trait Exporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli);
}

fn main() {
    let args = Cli::parse();

    let bytes = fs::read(&args.filepath).expect("Unable to read file");
    let parser = DJILog::from_bytes(bytes).expect("Unable to parse file");

    // Configure a decrypt method
    let decrypt_method = if parser.version >= 13 {
        if let Some(api_key) = &args.api_key {
            let keychains = parser
                .keychain_request()
                .expect("Unable to create keychain request")
                .fetch(api_key)
                .expect("Unable parse keychain result");
            DecryptMethod::Keychains(keychains)
        } else {
            panic!("Api Key required");
        }
    } else {
        DecryptMethod::None
    };

    let records = parser
        .records(decrypt_method.clone())
        .expect("Unable to parse records");

    let frames = parser
        .frames(decrypt_method)
        .expect("Unable to parse frames");

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
}
