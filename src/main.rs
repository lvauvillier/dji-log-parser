use clap::Parser;
use dji_log_parser::{DJILog, DecryptMethod};
use std::fs;

#[derive(Parser)]
struct Args {
    #[arg(value_name = "FILE")]
    filepath: String,

    #[arg(short, long)]
    api_key: Option<String>,
}

fn main() {
    let args = Args::parse();

    let bytes = fs::read(&args.filepath).expect("Unable to read file");
    let parser = DJILog::from_bytes(&bytes).expect("Unable to parse file");

    let decrypt_method = if parser.prefix.version >= 13 {
        if let Some(api_key) = args.api_key {
            DecryptMethod::ApiKey(api_key)
        } else {
            panic!("Api Key required");
        }
    } else {
        DecryptMethod::None
    };

    let records = parser
        .records(decrypt_method)
        .expect("Unable to parse records");
}
