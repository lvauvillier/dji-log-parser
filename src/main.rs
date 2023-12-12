use clap::Parser as ClapParser;
use dji_log_parser::Parser;
use std::fs;

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(value_name = "FILE")]
    filepath: String,

    #[arg(short, long)]
    sdk_key: Option<String>,
}

fn main() {
    let args = Args::parse();

    let bytes = fs::read(&args.filepath).expect("Unable to read file");
    let parser = Parser::from_bytes(&bytes).expect("Unable to parse file");

    if parser.version >= 13 && args.sdk_key.is_none() {
        panic!("A sdk_key is required for this log format");
    }

    println!("{:?}", parser);
}
