use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use crate::{DJILog, DecryptMethod, Frame};
use serde_json::json;

static mut LAST_ERROR: Option<String> = None;

fn set_last_error(error: String) {
    unsafe {
        LAST_ERROR = Some(error);
    }
}

fn frames_to_geojson(frames: &[Frame]) -> String {
    let feature_collection = json!({
        "type": "FeatureCollection",
        "features": frames.iter().map(|frame| {
            json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [frame.osd_longitude, frame.osd_latitude, frame.osd_altitude]
                },
                "properties": {
                    "time": frame.custom_date_time,
                    "height": frame.osd_height,
                    "speed": (frame.osd_x_speed.powi(2) + frame.osd_y_speed.powi(2) + frame.osd_z_speed.powi(2)).sqrt(),
                    // Add other properties as needed
                }
            })
        }).collect::<Vec<_>>()
    });
    serde_json::to_string_pretty(&feature_collection).unwrap()
}

#[no_mangle]
pub extern "C" fn parse_dji_log(input_path: *const c_char, api_key: *const c_char) -> bool {
    let input_path = unsafe { CStr::from_ptr(input_path).to_str().unwrap() };
    let api_key = unsafe { CStr::from_ptr(api_key).to_str().unwrap() };

    match std::fs::read(input_path) {
        Ok(bytes) => {
            match DJILog::from_bytes(bytes) {
                Ok(parser) => {
                    let decrypt_method = if parser.version >= 13 {
                        match parser.keychain_request() {
                            Ok(request) => match request.fetch(api_key) {
                                Ok(keychains) => DecryptMethod::Keychains(keychains),
                                Err(e) => {
                                    set_last_error(format!("Failed to fetch keychains: {}", e));
                                    return false;
                                }
                            },
                            Err(e) => {
                                set_last_error(format!("Failed to create keychain request: {}", e));
                                return false;
                            }
                        }
                    } else {
                        DecryptMethod::None
                    };

                    match parser.frames(decrypt_method) {
                        Ok(frames) => {
                            let geojson = frames_to_geojson(&frames);
                            let output_path = Path::new(input_path).with_extension("json");
                            if let Err(e) = std::fs::write(&output_path, geojson) {
                                set_last_error(format!("Failed to write GeoJSON: {}", e));
                                return false;
                            }
                            true
                        },
                        Err(e) => {
                            set_last_error(format!("Failed to parse frames: {}", e));
                            false
                        }
                    }
                }
                Err(e) => {
                    set_last_error(format!("Failed to parse DJI log: {}", e));
                    false
                }
            }
        }
        Err(e) => {
            set_last_error(format!("Failed to read file: {}", e));
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn get_last_error() -> *mut c_char {
    unsafe {
        LAST_ERROR
            .take()
            .map_or(std::ptr::null_mut(), |s| CString::new(s).unwrap().into_raw())
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            drop(CString::from_raw(s));
        }
    }
}

#[no_mangle]
pub extern "C" fn get_geojson_file_path(input_path: *const c_char) -> *mut c_char {
    let input_path = unsafe { CStr::from_ptr(input_path).to_str().unwrap() };
    let geojson_path = Path::new(input_path).with_extension("json");
    CString::new(geojson_path.to_str().unwrap()).unwrap().into_raw()
}