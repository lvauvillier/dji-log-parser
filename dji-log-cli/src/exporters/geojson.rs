use dji_log_parser::frame::Frame;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use geojson::{Feature, GeoJson, Geometry, JsonObject, JsonValue, Value};
use std::{fs::File, io::Write};

use crate::{Cli, Exporter};

pub struct GeoJsonExporter;

impl Exporter for GeoJsonExporter {
    fn export(&self, parser: &DJILog, _records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli) {
        if let Some(geojson_path) = &args.geojson {
            // Create a Value::LineString from all the coords.
            let mut coords = vec![];
            frames.iter().for_each(|frame| {
                let lat = frame.osd.latitude;
                let lon = frame.osd.longitude;
                let alt = frame.osd.altitude as f64;
                let coord = vec![lon, lat, alt];
                coords.push(coord);
            });
            let mut properties = JsonObject::new();
            let details = parser.details.clone();
            // Add details.subStreet, street, city as properties.
            properties.insert(
                "subStreet".to_string(),
                JsonValue::String(details.sub_street),
            );
            properties.insert("street".to_string(), JsonValue::String(details.street));
            properties.insert("city".to_string(), JsonValue::String(details.city));
            properties.insert("area".to_string(), JsonValue::String(details.area));
            properties.insert(
                "isFavorite".to_string(),
                JsonValue::Number(details.is_favorite.into()),
            );
            properties.insert(
                "isNew".to_string(),
                JsonValue::Number(details.is_new.into()),
            );
            properties.insert(
                "needsUpload".to_string(),
                JsonValue::Number(details.needs_upload.into()),
            );
            properties.insert(
                "recordLineCount".to_string(),
                JsonValue::Number(details.record_line_count.into()),
            );
            properties.insert(
                "detailInfoChecksum".to_string(),
                JsonValue::Number(details.detail_info_checksum.into()),
            );
            properties.insert(
                "startTime".to_string(),
                JsonValue::String(details.start_time.to_string()),
            );
            properties.insert(
                "totalDistance".to_string(),
                serde_json::Number::from_f64(details.total_distance.into())
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null),
            );
            properties.insert(
                "totalTime".to_string(),
                serde_json::Number::from_f64(details.total_time.into())
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null),
            );
            properties.insert(
                "maxHeight".to_string(),
                serde_json::Number::from_f64(details.max_height.into())
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null),
            );
            properties.insert(
                "maxHorizontalSpeed".to_string(),
                serde_json::Number::from_f64(details.max_horizontal_speed.into())
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null),
            );
            properties.insert(
                "maxVerticalSpeed".to_string(),
                serde_json::Number::from_f64(details.max_vertical_speed.into())
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null),
            );
            properties.insert(
                "captureNum".to_string(),
                JsonValue::Number(details.capture_num.into()),
            );
            properties.insert(
                "videoTime".to_string(),
                JsonValue::Number(details.video_time.into()),
            );
            properties.insert(
                "momentPicImageBufferLen".to_string(),
                JsonValue::Array(
                    details
                        .moment_pic_image_buffer_len
                        .iter()
                        .map(|x| JsonValue::Number((*x).into()))
                        .collect(),
                ),
            );
            properties.insert(
                "momentPicShrinkImageBufferLen".to_string(),
                JsonValue::Array(
                    details
                        .moment_pic_shrink_image_buffer_len
                        .iter()
                        .map(|x| JsonValue::Number((*x).into()))
                        .collect(),
                ),
            );
            // momentPicLongitude is an array of 4 f64s, so make sure to do the conversion like we do above.
            properties.insert(
                "momentPicLongitude".to_string(),
                JsonValue::Array(
                    details
                        .moment_pic_longitude
                        .iter()
                        .map(|x| {
                            serde_json::Number::from_f64((*x).into())
                                .map(JsonValue::Number)
                                .unwrap_or(JsonValue::Null)
                        })
                        .collect(),
                ),
            );
            properties.insert(
                "momentPicLatitude".to_string(),
                JsonValue::Array(
                    details
                        .moment_pic_latitude
                        .iter()
                        .map(|x| {
                            serde_json::Number::from_f64((*x).into())
                                .map(JsonValue::Number)
                                .unwrap_or(JsonValue::Null)
                        })
                        .collect(),
                ),
            );
            properties.insert(
                "takeOffAltitude".to_string(),
                serde_json::Number::from_f64(details.take_off_altitude.into())
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null),
            );
            let product_type = serde_json::to_string(&details.product_type).unwrap();
            properties.insert(
                "productType".to_string(),
                // product_type is an enum. Serialize it to JSON.
                JsonValue::String(product_type),
            );
            properties.insert(
                "aircraftName".to_string(),
                JsonValue::String(details.aircraft_name),
            );
            properties.insert(
                "aircraftSN".to_string(),
                JsonValue::String(details.aircraft_sn),
            );
            properties.insert("cameraSN".to_string(), JsonValue::String(details.camera_sn));

            let geometry = Geometry::new(Value::LineString(coords));
            let feature = Feature {
                bbox: None,
                geometry: Some(geometry),
                id: None,
                properties: Some(properties),
                foreign_members: None,
            };
            let geojson = GeoJson::Feature(feature);
            let geojson_string = geojson.to_string();
            let mut file = File::create(geojson_path).expect("Unable to create GeoJSON file");
            file.write_all(geojson_string.as_bytes())
                .expect("Unable to write GeoJSON data");
        }
    }
}
