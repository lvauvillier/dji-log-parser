use clap::Parser;
use dji_log_parser::DJILog;
use std::fs;

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "FILE")]
    filepath: String,

    #[arg(short, long)]
    sdk_key: Option<String>,
}

fn main() {
    let args = Args::parse();

    let bytes = fs::read(&args.filepath).expect("Unable to read file");
    let parser = DJILog::from_bytes(&bytes).expect("Unable to parse file");

    if parser.version >= 13 && args.sdk_key.is_none() {
        panic!("A sdk_key is required for this log format");
    }

    println!("{:?}", parser);
}
