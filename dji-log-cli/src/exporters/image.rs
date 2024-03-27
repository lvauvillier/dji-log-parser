use chrono::{DateTime, Utc};
use dji_log_parser::layout::info::ProductType;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use exif::experimental::Writer;
use exif::{Field, In, Rational, Tag, Value};
use img_parts::jpeg::Jpeg;
use img_parts::{Bytes, ImageEXIF};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::utils::decimal_to_dms;
use crate::{Cli, Exporter};

struct ExifInfo {
    datetime: DateTime<Utc>,
    latitude: f64,
    longitude: f64,
    model: ProductType,
}

pub struct ImageExporter;

impl Exporter for ImageExporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, args: &Cli) {
        // Export Images
        if let Some(image_path) = &args.images {
            let mut index = 0;
            records
                .iter()
                .filter(|r| matches!(r, Record::JPEG(_)))
                .for_each(|record| {
                    if let Record::JPEG(data) = record {
                        if index < 4
                            && parser.info.moment_pic_image_buffer_len[index] == data.len() as i32
                        {
                            let file_name = image_path.replace("%d", &(index + 1).to_string());
                            self.save_image_with_exif_metadata(
                                data,
                                file_name,
                                ExifInfo {
                                    datetime: parser.info.start_time,
                                    latitude: if parser.info.moment_pic_latitude[index] != 0.0 {
                                        parser.info.moment_pic_latitude[index]
                                    } else {
                                        parser.info.latitude
                                    },
                                    longitude: if parser.info.moment_pic_longitude[index] != 0.0 {
                                        parser.info.moment_pic_longitude[index]
                                    } else {
                                        parser.info.longitude
                                    },
                                    model: parser.info.product_type,
                                },
                            );
                            index += 1;
                        }
                    }
                });
        }

        // Export Thumbnails
        if let Some(thumbnails_path) = &args.thumbnails {
            let mut index = 0;
            records
                .iter()
                .filter(|r| matches!(r, Record::JPEG(_)))
                .for_each(|record| {
                    if let Record::JPEG(data) = record {
                        if index < 4
                            && parser.info.moment_pic_shrink_image_buffer_len[index]
                                == data.len() as i32
                        {
                            let file_name = thumbnails_path.replace("%d", &(index + 1).to_string());
                            self.save_image_with_exif_metadata(
                                data,
                                file_name,
                                ExifInfo {
                                    datetime: parser.info.start_time,
                                    latitude: if parser.info.moment_pic_latitude[index] != 0.0 {
                                        parser.info.moment_pic_latitude[index]
                                    } else {
                                        parser.info.latitude
                                    },
                                    longitude: if parser.info.moment_pic_longitude[index] != 0.0 {
                                        parser.info.moment_pic_longitude[index]
                                    } else {
                                        parser.info.longitude
                                    },
                                    model: parser.info.product_type,
                                },
                            );
                            index += 1;
                        }
                    }
                });
        }
    }
}

impl ImageExporter {
    fn save_image_with_exif_metadata(&self, data: &Vec<u8>, file_name: String, info: ExifInfo) {
        let jpeg_result = Jpeg::from_bytes(Bytes::copy_from_slice(data));

        if let Err(_) = jpeg_result {
            // Don't add exif metadata if JPEG creation fails
            let mut file = File::create(Path::new(&file_name)).unwrap();
            file.write_all(data).unwrap();
            return;
        }

        let mut jpeg = jpeg_result.unwrap();
        let mut writer = Writer::new();

        // Set Latitude
        let (degrees, minutes, seconds) = decimal_to_dms(info.latitude.abs());
        let latitude_field = Field {
            tag: Tag::GPSLatitude,
            ifd_num: In::PRIMARY,
            value: Value::Rational(vec![
                Rational::from((degrees as u32, 1)),
                Rational::from((minutes as u32, 1)),
                Rational::from((seconds as u32, 1)),
            ]),
        };
        writer.push_field(&latitude_field);

        let latitude_ref_field = Field {
            tag: Tag::GPSLatitudeRef,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![(if info.latitude >= 0.0 { "N" } else { "S" })
                .to_string()
                .into_bytes()]),
        };
        writer.push_field(&latitude_ref_field);

        // Set Longitude
        let (degrees, minutes, seconds) = decimal_to_dms(info.longitude.abs());
        let longitude_field = Field {
            tag: Tag::GPSLongitude,
            ifd_num: In::PRIMARY,
            value: Value::Rational(vec![
                Rational::from((degrees as u32, 1)),
                Rational::from((minutes as u32, 1)),
                Rational::from((seconds as u32, 1)),
            ]),
        };
        writer.push_field(&longitude_field);

        let longitude_ref_field = Field {
            tag: Tag::GPSLongitudeRef,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![(if info.longitude >= 0.0 { "E" } else { "W" })
                .to_string()
                .into_bytes()]),
        };
        writer.push_field(&longitude_ref_field);

        // Set Datetime
        let datetime = Field {
            tag: Tag::DateTime,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![info
                .datetime
                .format("%Y:%m:%d %H:%M:%S")
                .to_string()
                .into_bytes()]),
        };
        writer.push_field(&datetime);

        // Set Datetime Original
        let datetime_original = Field {
            tag: Tag::DateTimeOriginal,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![info
                .datetime
                .format("%Y:%m:%d %H:%M:%S")
                .to_string()
                .into_bytes()]),
        };
        writer.push_field(&datetime_original);

        // Set Make
        let make = Field {
            tag: Tag::Make,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec!["DJI".into()]),
        };
        writer.push_field(&make);

        // Set Model
        let model_name = serde_json::to_string(&info.model).unwrap();
        let model = Field {
            tag: Tag::Model,
            ifd_num: In::PRIMARY,
            value: Value::Ascii(vec![(if model_name.len() > 2 {
                model_name[1..model_name.len() - 1].to_string()
            } else {
                model_name
            })
            .into()]),
        };
        writer.push_field(&model);

        let mut buf = std::io::Cursor::new(Vec::new());
        writer.write(&mut buf, false).unwrap();

        jpeg.set_exif(Some(Bytes::from(buf.into_inner())));

        let file = File::create(Path::new(&file_name)).expect("Unable to create image file");
        jpeg.encoder()
            .write_to(file)
            .expect("Unable to write image file");
    }
}
