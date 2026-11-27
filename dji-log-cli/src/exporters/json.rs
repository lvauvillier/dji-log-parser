use dji_log_parser::frame::Frame;
use dji_log_parser::frame::FrameDetails;
use dji_log_parser::layout::details::Details;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use serde::Serialize;

use std::io::Write;

use crate::{write_file, Cli, ExportResult, Exporter};

#[derive(Serialize, Debug)]
struct RecordJsonData<'a> {
    version: u8,
    details: Details,
    records: Vec<&'a Record>,
}

#[derive(Serialize, Debug)]
struct FrameJsonData<'a> {
    version: u8,
    details: FrameDetails,
    frames: &'a [Frame],
}

pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn export(
        &self,
        parser: &DJILog,
        records: &[Record],
        frames: &[Frame],
        args: &Cli,
    ) -> ExportResult {
        let json_data = if args.raw {
            serde_json::to_string(&RecordJsonData {
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
            })?
        } else {
            serde_json::to_string(&FrameJsonData {
                version: parser.version,
                details: parser.details.clone().into(),
                frames,
            })?
        };

        if let Some(output_path) = &args.output {
            write_file(output_path, json_data.as_bytes())?;
        } else {
            let mut stdout = std::io::stdout().lock();
            let written = stdout
                .write_all(json_data.as_bytes())
                .and_then(|()| stdout.write_all(b"\n"));
            // A closed output, such as a pipe to head, isn't an error
            match written {
                Err(error) if error.kind() != std::io::ErrorKind::BrokenPipe => {
                    return Err(error.into())
                }
                _ => {}
            }
        }
        Ok(())
    }
}
