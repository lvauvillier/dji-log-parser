use csv::Writer;
use dji_log_parser::frame::Frame;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;

use crate::{Cli, Exporter};

#[derive(Default)]
pub struct CSVExporter;

impl Exporter for CSVExporter {
    fn export(&self, _parser: &DJILog, _records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli) {
        if let Some(csv_path) = &args.csv {
            let mut writer = Writer::from_path(csv_path).unwrap();

            frames
                .iter()
                .for_each(|frame| writer.serialize(&frame).unwrap())
        }
    }
}
