use clap::Parser;
use dji_log_parser::record::Record;
use dji_log_parser::{DJILog, DecryptMethod, Info};
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input log file
    #[arg(value_name = "FILE")]
    filepath: String,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,

    /// Image file path.
    #[arg(short, long)]
    images: Option<String>,

    /// DJI keychain Api Key
    #[arg(short, long)]
    api_key: Option<String>,
}

#[derive(Serialize, Debug)]
struct ExtractedData<'a> {
    version: u8,
    info: Info,
    records: Vec<&'a Record>,
}

fn main() {
    let args = Cli::parse();

    let bytes = fs::read(&args.filepath).expect("Unable to read file");
    let parser = DJILog::from_bytes(&bytes).expect("Unable to parse file");

    // Configure decrypt method
    let decrypt_method = if parser.version >= 13 {
        if let Some(api_key) = args.api_key {
            DecryptMethod::ApiKey(api_key)
        } else {
            panic!("Api Key required");
        }
    } else {
        DecryptMethod::None
    };

    // Get records
    let records: Vec<Record> = parser
        .records(decrypt_method)
        .expect("Unable to parse records");

    // Export data
    let extracted_data = ExtractedData {
        version: parser.version,
        info: parser.info.clone(),
        records: records
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    Record::KeyStorage(_)
                        | Record::Unknown(_, _)
                        | Record::Invalid(_)
                        | Record::JPEG(_)
                )
            })
            .collect(),
    };

    let json_data = serde_json::to_string(&extracted_data).unwrap();

    if let Some(output_path) = args.output {
        let mut file = File::create(output_path).expect("Unable to create output file");
        file.write_all(json_data.as_bytes())
            .expect("Unable to write data");
    } else {
        println!("{json_data}");
    }

    // Export Images
    if let Some(image_path) = args.images {
        records
            .iter()
            .filter(|r| matches!(r, Record::JPEG(_)))
            .enumerate()
            .for_each(|(i, record)| {
                if let Record::JPEG(data) = record {
                    let file_name = image_path.replace("%d", &(i + 1).to_string());
                    let mut file = File::create(Path::new(&file_name)).unwrap();
                    file.write_all(data).unwrap();
                }
            });
    }
}
