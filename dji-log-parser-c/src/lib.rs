use dji_log_parser::frame::Frame;
use dji_log_parser::layout::details::Details;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use dji_log_parser::Error;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;
use std::sync::Mutex;

#[derive(Serialize, Debug)]
struct JsonData<'a> {
    version: u8,
    details: Details,
    records: Vec<&'a Record>,
    frames: &'a Vec<Frame>,
}

static LAST_ERROR: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub(crate) fn set_last_error(error: String) {
    let mut last_error = LAST_ERROR.lock().unwrap();
    *last_error = Some(error);
}

#[no_mangle]
pub extern "C" fn c_api_free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            drop(CString::from_raw(s));
        }
    }
}

fn get_last_error() -> String {
    LAST_ERROR.lock().unwrap().take().unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn get_error() -> *mut c_char {
    let error = crate::get_last_error();
    match CString::new(error) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn parse_from_bytes(
    bytes: *const u8,
    length: usize,
    api_key: *const c_char,
) -> *mut c_char {
    let input_bytes = unsafe { slice::from_raw_parts(bytes, length) };
    let api_key = unsafe { CStr::from_ptr(api_key).to_str().unwrap() };

    match process_dji_log(input_bytes, api_key) {
        Ok(result) => CString::new(result).unwrap().into_raw(),
        Err(e) => {
            set_last_error(format!("Failed to process DJI log: {}", e));
            std::ptr::null_mut()
        }
    }
}

fn to_json(parser: &DJILog, records: &Vec<Record>, frames: &Vec<Frame>) -> String {
    let json_data = JsonData {
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
        frames,
    };

    serde_json::to_string(&json_data).unwrap()
}

fn process_dji_log(bytes: &[u8], api_key: &str) -> Result<String, Error> {
    let parser = DJILog::from_bytes(bytes.to_vec())?;

    let decrypt_method = if parser.version >= 13 {
        let keychains = parser.fetch_keychains(api_key)?;
        Some(keychains)
    } else {
        None
    };

    let records = parser.records(decrypt_method.clone())?;

    let frames = parser.frames(decrypt_method)?;

    Ok(to_json(&parser, &records, &frames))
}
