use clap::Parser;
use dji_log_parser::record::Record;
use dji_log_parser::{DJILog, DecryptMethod, Info};
use geojson::{Feature, GeoJson, Geometry, JsonObject, JsonValue, Value};
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

    /// Thumbnail file path.
    #[arg(short, long)]
    thumbnails: Option<String>,

    // GeoJSON file path.
    #[arg(short, long)]
    geojson: Option<String>,

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
        let number_of_images = parser
            .info
            .moment_pic_image_buffer_len
            .iter()
            .filter(|&&x| x != 0)
            .count();

        if number_of_images > 0 {
            records
                .iter()
                .filter(|r| matches!(r, Record::JPEG(_)))
                .enumerate()
                .for_each(|(i, record)| {
                    if i < number_of_images {
                        if let Record::JPEG(data) = record {
                            let file_name = image_path.replace("%d", &(i + 1).to_string());
                            let mut file = File::create(Path::new(&file_name)).unwrap();
                            file.write_all(data).unwrap();
                        }
                    }
                });
        }
    }

    // Export Thumbnails
    if let Some(thumbnails_path) = args.thumbnails {
        let number_of_images = parser
            .info
            .moment_pic_image_buffer_len
            .iter()
            .filter(|&&x| x != 0)
            .count();

        if number_of_images > 0 {
            records
                .iter()
                .filter(|r| matches!(r, Record::JPEG(_)))
                .enumerate()
                .for_each(|(i, record)| {
                    if i >= number_of_images {
                        if let Record::JPEG(data) = record {
                            let file_name = thumbnails_path
                                .replace("%d", &(i - number_of_images + 1).to_string());
                            let mut file = File::create(Path::new(&file_name)).unwrap();
                            file.write_all(data).unwrap();
                        }
                    }
                });
        }
    }

    // Export GeoJSON
    if let Some(geojson_path) = args.geojson {
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
            JsonValue::Number(serde_json::Number::from_f64(info.total_distance.into()).unwrap()),
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
                    .map(|x| JsonValue::Number(serde_json::Number::from_f64((*x).into()).unwrap()))
                    .collect(),
            ),
        );
        properties.insert(
            "momentPicLatitude".to_string(),
            JsonValue::Array(
                info.moment_pic_latitude
                    .iter()
                    .map(|x| JsonValue::Number(serde_json::Number::from_f64((*x).into()).unwrap()))
                    .collect(),
            ),
        );
        properties.insert(
            "takeOffAltitude".to_string(),
            JsonValue::Number(serde_json::Number::from_f64(info.take_off_altitude.into()).unwrap()),
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
