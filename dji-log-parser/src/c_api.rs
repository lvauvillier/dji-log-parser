use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::{DJILog, DecryptMethod, set_last_error };

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
                        Ok(_frames) => {
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
    let error = crate::get_last_error();
    match CString::new(error) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn c_api_free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            drop(CString::from_raw(s));
        }
    }
}