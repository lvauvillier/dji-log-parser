use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;
use crate::{DJILog, DecryptMethod, set_last_error};

#[no_mangle]
pub extern "C" fn parse_dji_log(input_path: *const c_char, api_key: *const c_char) -> bool {
    let input_path = unsafe { CStr::from_ptr(input_path).to_str().unwrap() };
    let api_key = unsafe { CStr::from_ptr(api_key).to_str().unwrap() };

    match std::fs::read(input_path) {
        Ok(bytes) => {
            match DJILog::from_bytes(bytes) {
                Ok(parser) => {
                    let decrypt_method = if parser.version >= 13 {
                        DecryptMethod::ApiKey(api_key.to_string())
                    } else {
                        DecryptMethod::None
                    };

                    match parser.frames(decrypt_method) {
                        Ok(frames) => {
                            let geojson = DJILog::frames_to_geojson(&frames);
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
pub extern "C" fn get_geojson_file_path(input_path: *const c_char) -> *mut c_char {
    let input_path = unsafe { CStr::from_ptr(input_path).to_str().unwrap() };
    let geojson_path = Path::new(input_path).with_extension("json");
    CString::new(geojson_path.to_str().unwrap()).unwrap().into_raw()
}