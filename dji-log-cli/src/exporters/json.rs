use dji_log_parser::frame::Frame;
use dji_log_parser::layout::details::Details;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

use crate::{Cli, Exporter};

#[derive(Serialize, Debug)]
struct JsonData<'a> {
    version: u8,
    details: Details,
    records: Vec<&'a Record>,
    frames: &'a Vec<Frame>,
}

pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli) {
        let json_data = JsonData {
            version: parser.version,
            details: parser.details.clone(),
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
            frames,
        };

        let json_data = serde_json::to_string(&json_data).unwrap();

        if let Some(output_path) = &args.output {
            let mut file = File::create(output_path).expect("Unable to create output file");
            file.write_all(json_data.as_bytes())
                .expect("Unable to write data");
        } else {
            println!("{json_data}");
        }
    }
}
