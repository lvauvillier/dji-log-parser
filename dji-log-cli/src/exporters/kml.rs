use dji_log_parser::frame::Frame;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use kml::types::{AltitudeMode, Coord, Geometry, LineString, Placemark};
use kml::{Kml, KmlDocument, KmlVersion, KmlWriter};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::{Cli, Exporter};

pub struct KmlExporter;

impl Exporter for KmlExporter {
    fn export(&self, parser: &DJILog, _records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli) {
        if let Some(kml_path) = &args.kml {
            let mut coords = vec![];

            frames.iter().for_each(|frame| {
                let coord = Coord {
                    x: frame.osd.longitude,
                    y: frame.osd.latitude,
                    z: Some(frame.osd.altitude as f64),
                };
                coords.push(coord);
            });

            let mut document_attrs = HashMap::new();
            document_attrs.insert(
                "xmlns".to_owned(),
                "http://www.opengis.net/kml/2.2".to_owned(),
            );
            document_attrs.insert(
                "xmlns:gx".to_owned(),
                "http://www.google.com/kml/ext/2.2".to_owned(),
            );
            document_attrs.insert(
                "xmlns:kml".to_owned(),
                "http://www.opengis.net/kml/2.2".to_owned(),
            );
            document_attrs.insert(
                "xmlns:atom".to_owned(),
                "http://www.w3.org/2005/Atom".to_owned(),
            );

            let document = KmlDocument::<f64> {
                version: KmlVersion::V22,
                attrs: document_attrs,
                elements: vec![Kml::Placemark(Placemark {
                    name: Some(parser.details.aircraft_name.to_string()),
                    geometry: Some(Geometry::LineString(LineString {
                        coords,
                        altitude_mode: AltitudeMode::RelativeToGround,
                        ..LineString::default()
                    })),
                    ..Placemark::default()
                })],
            };

            let kml = Kml::KmlDocument(document);

            let mut buf = Vec::new();
            let mut writer = KmlWriter::from_writer(&mut buf);
            writer.write(&kml).unwrap();

            let mut file = File::create(kml_path).expect("Unable to create KML file");
            file.write_all(&buf).expect("Unable to write KML data");
        }
    }
}
