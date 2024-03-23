use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use geojson::{Feature, GeoJson, Geometry, JsonObject, JsonValue, Value};
use std::{fs::File, io::Write};

use crate::{Cli, Exporter};

pub struct GeoJsonExporter;

impl Exporter for GeoJsonExporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, args: &Cli) {
        if let Some(geojson_path) = &args.geojson {
            // Create a Value::LineString from all the coords.
            let mut coords = vec![];
            records
                .iter()
                .filter(|r| matches!(r, Record::OSD(_)))
                .for_each(|r| {
                    if let Record::OSD(osd) = r {
                        let lat = osd.latitude;
                        let lon = osd.longitude;
                        let alt = osd.altitude as f64;
                        let coord = vec![lon, lat, alt];
                        coords.push(coord);
                    }
                });
            let mut properties = JsonObject::new();
            let info = parser.info.clone();
            // Add info.subStreet, street, city as properties.
            properties.insert("subStreet".to_string(), JsonValue::String(info.sub_street));
            properties.insert("street".to_string(), JsonValue::String(info.street));
            properties.insert("city".to_string(), JsonValue::String(info.city));
            properties.insert("area".to_string(), JsonValue::String(info.area));
            properties.insert(
                "isFavorite".to_string(),
                JsonValue::Number(info.is_favorite.into()),
            );
            properties.insert("isNew".to_string(), JsonValue::Number(info.is_new.into()));
            properties.insert(
                "needsUpload".to_string(),
                JsonValue::Number(info.needs_upload.into()),
            );
            properties.insert(
                "recordLineCount".to_string(),
                JsonValue::Number(info.record_line_count.into()),
            );
            properties.insert(
                "detailInfoChecksum".to_string(),
                JsonValue::Number(info.detail_info_checksum.into()),
            );
            properties.insert(
                "startTime".to_string(),
                JsonValue::String(info.start_time.to_string()),
            );
            properties.insert(
                "totalDistance".to_string(),
                JsonValue::Number(
                    serde_json::Number::from_f64(info.total_distance.into()).unwrap(),
                ),
            );
            properties.insert(
                "totalTime".to_string(),
                JsonValue::Number(serde_json::Number::from_f64(info.total_time.into()).unwrap()),
            );
            properties.insert(
                "maxHeight".to_string(),
                JsonValue::Number(serde_json::Number::from_f64(info.max_height.into()).unwrap()),
            );
            properties.insert(
                "maxHorizontalSpeed".to_string(),
                JsonValue::Number(
                    serde_json::Number::from_f64(info.max_horizontal_speed.into()).unwrap(),
                ),
            );
            properties.insert(
                "maxVerticalSpeed".to_string(),
                JsonValue::Number(
                    serde_json::Number::from_f64(info.max_vertical_speed.into()).unwrap(),
                ),
            );
            properties.insert(
                "captureNum".to_string(),
                JsonValue::Number(info.capture_num.into()),
            );
            properties.insert(
                "videoTime".to_string(),
                JsonValue::Number(info.video_time.into()),
            );
            properties.insert(
                "momentPicImageBufferLen".to_string(),
                JsonValue::Array(
                    info.moment_pic_image_buffer_len
                        .iter()
                        .map(|x| JsonValue::Number((*x).into()))
                        .collect(),
                ),
            );
            properties.insert(
                "momentPicShrinkImageBufferLen".to_string(),
                JsonValue::Array(
                    info.moment_pic_shrink_image_buffer_len
                        .iter()
                        .map(|x| JsonValue::Number((*x).into()))
                        .collect(),
                ),
            );
            // momentPicLongitude is an array of 4 f64s, so make sure to do the conversion like we do above.
            properties.insert(
                "momentPicLongitude".to_string(),
                JsonValue::Array(
                    info.moment_pic_longitude
                        .iter()
                        .map(|x| {
                            JsonValue::Number(serde_json::Number::from_f64((*x).into()).unwrap())
                        })
                        .collect(),
                ),
            );
            properties.insert(
                "momentPicLatitude".to_string(),
                JsonValue::Array(
                    info.moment_pic_latitude
                        .iter()
                        .map(|x| {
                            JsonValue::Number(serde_json::Number::from_f64((*x).into()).unwrap())
                        })
                        .collect(),
                ),
            );
            properties.insert(
                "takeOffAltitude".to_string(),
                JsonValue::Number(
                    serde_json::Number::from_f64(info.take_off_altitude.into()).unwrap(),
                ),
            );
            let product_type = serde_json::to_string(&info.product_type).unwrap();
            properties.insert(
                "productType".to_string(),
                // product_type is an enum. Serialize it to JSON.
                JsonValue::String(product_type),
            );
            properties.insert(
                "aircraftName".to_string(),
                JsonValue::String(info.aircraft_name),
            );
            properties.insert(
                "aircraftSN".to_string(),
                JsonValue::String(info.aircraft_sn),
            );
            properties.insert("cameraSN".to_string(), JsonValue::String(info.camera_sn));

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
