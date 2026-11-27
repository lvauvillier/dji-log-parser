use clap::Parser;
use dji_log_parser::frame::{records_to_frames, Frame};
use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::layout::auxiliary::Department;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use exporters::{CSVExporter, GeoJsonExporter, ImageExporter, JsonExporter, KmlExporter};
use std::error::Error;
use std::fs;
use std::process::ExitCode;

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

    /// DJI keychain Api Key, read from the DJI_API_KEY environment variable when not given
    #[arg(short, long, env = "DJI_API_KEY", hide_env_values = true)]
    api_key: Option<String>,

    /// Read the keychains from FILE instead of fetching them with the Api Key
    #[arg(long, value_name = "FILE")]
    keychains: Option<String>,

    /// Write the keychains fetched with the Api Key to FILE, to read them later with --keychains
    #[arg(long, value_name = "FILE")]
    save_keychains: Option<String>,

    /// Extract raw records instead of normalized frames
    #[arg(short, long)]
    raw: bool,

    /// Custom department for keychain request
    #[arg(long)]
    api_custom_department: Option<u8>,

    /// Custom version for keychain request
    #[arg(long)]
    api_custom_version: Option<u16>,
}

pub(crate) type ExportResult = Result<(), Box<dyn Error>>;

pub(crate) trait Exporter {
    fn export(
        &self,
        parser: &DJILog,
        records: &[Record],
        frames: &[Frame],
        args: &Cli,
    ) -> ExportResult;
}

/// Writes a file, with its path in the error
pub(crate) fn write_file(path: &str, data: &[u8]) -> ExportResult {
    fs::write(path, data).map_err(|error| format!("unable to write {path}: {error}"))?;
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Cli) -> Result<(), Box<dyn Error>> {
    let bytes = fs::read(&args.filepath)
        .map_err(|error| format!("unable to read {}: {error}", args.filepath))?;
    let parser = DJILog::from_bytes(bytes)
        .map_err(|error| format!("unable to parse {}: {error}", args.filepath))?;

    let keychains = if parser.version >= 13 {
        Some(keychains(&parser, &args)?)
    } else {
        None
    };

    let records = parser.records(keychains)?;

    // Built from the records above rather than with parser.frames(), which would parse the log again
    let frames = records_to_frames(&records, parser.details.clone());

    let exporters: Vec<Box<dyn Exporter>> = vec![
        Box::new(JsonExporter),
        Box::new(ImageExporter),
        Box::new(GeoJsonExporter),
        Box::new(KmlExporter),
        Box::new(CSVExporter),
    ];

    for exporter in exporters {
        exporter.export(&parser, &records, &frames, &args)?;
    }
    Ok(())
}

/// Keychains of a log of version 13 or later: read from a file, or fetched from the DJI API
fn keychains(
    parser: &DJILog,
    args: &Cli,
) -> Result<Vec<Vec<KeychainFeaturePoint>>, Box<dyn Error>> {
    if let Some(path) = &args.keychains {
        let data = fs::read(path).map_err(|error| format!("unable to read {path}: {error}"))?;
        return serde_json::from_slice(&data)
            .map_err(|error| format!("invalid keychains in {path}: {error}").into());
    }

    let api_key = args.api_key.as_deref().ok_or(
        "an Api Key (--api-key or DJI_API_KEY) or a keychains file (--keychains) is required \
         for logs of version 13 and above",
    )?;
    let keychains = fetch_keychains(parser, api_key, args)?;

    if let Some(path) = &args.save_keychains {
        write_file(path, serde_json::to_string_pretty(&keychains)?.as_bytes())?;
    }
    Ok(keychains)
}

/// Fetches the keychains with the department of the log, or the one given. When the department
/// of the log is rejected, tries the departments of the DJI apps.
fn fetch_keychains(
    parser: &DJILog,
    api_key: &str,
    args: &Cli,
) -> Result<Vec<Vec<KeychainFeaturePoint>>, Box<dyn Error>> {
    let department = args.api_custom_department.map(Department::from);
    let version = args.api_custom_version;
    let fetch = |department| {
        parser
            .keychains_request_with_custom_params(department, version)?
            .fetch(api_key, None)
    };

    let is_custom = department.is_some();
    match fetch(department) {
        Err(error) if !is_custom && is_invalid_ciphertext(&error) => {
            for department in [Department::DJIFly, Department::DJIGO, Department::DJIPilot] {
                match fetch(Some(department)) {
                    Err(error) if is_invalid_ciphertext(&error) => continue,
                    result => return Ok(result?),
                }
            }
            Err(format!("unable to fetch the keychains: {error}").into())
        }
        result => Ok(result.map_err(|error| format!("unable to fetch the keychains: {error}"))?),
    }
}

/// The error of the DJI API when the department of the request doesn't match the log
fn is_invalid_ciphertext(error: &dji_log_parser::Error) -> bool {
    matches!(error, dji_log_parser::Error::ApiError(message) if message == "invalid ciphertext")
}
