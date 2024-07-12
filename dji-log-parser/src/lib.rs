//! # DJILog Parser Module
//!
//! This module provides functionality for parsing DJI log files.
//!
//! ## Encryption in Version 13 and Later
//! Starting with version 13, log records are AES encrypted and require a specific keychain
//! for decryption. This keychain must be obtained from DJI using their API. An apiKey is
//! necessary to access the DJI API.
//!
//! ### Obtaining an ApiKey
//! To acquire an apiKey, follow these steps:
//! 1. Visit [DJI Developer Technologies](https://developer.dji.com/user) and log in.
//! 2. Click `CREATE APP`, choose `Open API` as the App Type, and provide the necessary
//!    details like `App Name`, `Category`, and `Description`.
//! 3. After creating the app, activate it through the link sent to your email.
//! 4. On your developer user page, find your app's details to retrieve the ApiKey
//!    (labeled as the SDK key).
//!
//! ## Usage
//!
//! ### Initialization
//! Initialize a `DJILog` instance from a byte slice to access version information and metadata:
//! ```
//! let parser = DJILog::from_bytes(&bytes).unwrap();
//! println!("Version: {:?}", parser.version);
//! println!("Details: {:?}", parser.details);
//! ```
//!
//! ### Accessing raw Records
//! Decrypt raw records based on the log file version.
//!
//! For versions prior to 13:
//! ```rust
//! let records = parser.records(DecryptMethod::None);
//! ```
//!
//! For version 13 and later:
//! ```rust
//! let records = parser.records(DecryptMethod::ApiKey("__DJI_API_KEY__"));
//! ```
//!
//! ### Accessing Frames
//! Decrypt frames based on the log file version.
//!
//! A `Frame` is a standardized representation of log data, normalized across
//! different log versions. It provides a consistent and easy-to-use format
//! for analyzing and processing DJI log information.
//!
//! For versions prior to 13:
//! ```rust
//! let frames = parser.frames(DecryptMethod::None);
//! ```
//!
//! For version 13 and later:
//! ```rust
//! let frames = parser.frames(DecryptMethod::ApiKey("__DJI_API_KEY__"));
//! ```
//!
//!
//! ### Advanced: Manual Keychain Retrieval
//! For scenarios like caching, multiple calls, offline use, or custom server communication,
//! the library exposes the internal keychain retrieval process:
//! ```rust
//! // We want to get both records and frames.
//! // We manually retrieve keychains once to avoid running two network requests.
//! let keychain_request = parser.keychain_request().unwrap();
//! let keychains = keychain_request.fetch("__DJI_API_KEY__").unwrap();
//!
//! let records = parser.records(DecryptMethod::Keychains(keychains.clone()));
//! let frames = parser.frames(DecryptMethod::Keychains(keychains));
//! ```
//!
//! Note: Replace `__DJI_API_KEY__` with your actual apiKey.
//!
//!
//! ## Binary structure of log files:
//!
//! v1 -> v6
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │     Records     │                │
//! ├─────────────────┤<───────────────┘
//! │     Details     │ detail_length
//! └─────────────────┘
//! ```
//!
//! v7 -> v11
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │     Records     │                │
//! │   (Encrypted)   │                |
//! ├─────────────────┤<───────────────┘
//! │     Details     │ detail_length
//! └─────────────────┘
//!```
//!
//! v12
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │      Details    │ detail_length  │
//! ├─────────────────┤                │
//! │     Records     │                │
//! │   (Encrypted)   │                │
//! └─────────────────┘<───────────────┘
//!```
//!
//! v13 -> v14
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │ Auxiliary Info  |                |
//! │ (Encrypted      │ detail_length  │
//! │      Details)   |                |
//! ├─────────────────┤                │
//! │    Auxiliary    |                |
//! │     Version     |                │
//! ├─────────────────┤<───────────────┘
//! │     Records     │
//! │(Encrypted + AES)|
//! └─────────────────┘
//! ```
use base64::engine::general_purpose::STANDARD as Base64Standard;
use base64::Engine as _;
use binrw::io::Cursor;
use binrw::BinRead;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use thiserror::Error;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;
use serde_json::json;
use std::sync::Mutex;
use once_cell::sync::Lazy;

mod decoder;
pub mod frame;
pub mod keychain;
pub mod layout;
pub mod record;
pub mod c_api;
mod utils;

use frame::{records_to_frames, Frame};
use keychain::{Keychain, KeychainCipherText, KeychainRequest};
use layout::auxiliary::Auxiliary;
use layout::details::Details;
use layout::prefix::Prefix;
use record::Record;
use log::{debug, error};

use crate::utils::pad_with_zeros;

static LAST_ERROR: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub(crate) fn set_last_error(error: String) {
    let mut last_error = LAST_ERROR.lock().unwrap();
    *last_error = Some(error);
}

#[derive(PartialEq, Debug, Error)]
#[non_exhaustive]
pub enum DJILogError {
    #[error("Failed to parse prefix: {0}")]
    PrefixParseError(String),

    #[error("Failed to parse detail: {0}")]
    DetailsParseError(String),

    #[error("Failed to parse auxiliary block: {0}")]
    AuxiliaryParseError(String),

    #[error("Failed to parse record: {0}")]
    RecordParseError(String),

    #[error("Failed to parse keychain: {0}")]
    KeychainParseError(String),

    #[error("Failed serialize object: {0}")]
    SerializeError(String),

    #[error("Deserialize error: {0}")]
    DeserializeError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

#[no_mangle]
pub extern "C" fn get_geojson_string(input_path: *const c_char, api_key: *const c_char) -> *mut c_char {
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
                            CString::new(geojson).unwrap().into_raw()
                        },
                        Err(e) => {
                            set_last_error(format!("Failed to parse frames: {}", e));
                            std::ptr::null_mut()
                        }
                    }
                }
                Err(e) => {
                    set_last_error(format!("Failed to parse DJI log: {}", e));
                    std::ptr::null_mut()
                }
            }
        }
        Err(e) => {
            set_last_error(format!("Failed to read file: {}", e));
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn get_geojson_string_from_bytes(bytes: *const u8, length: usize, api_key: *const c_char) -> *mut c_char {
    let input_bytes = unsafe { slice::from_raw_parts(bytes, length) };
    let api_key = unsafe { CStr::from_ptr(api_key).to_str().unwrap() };

    match process_dji_log(input_bytes, api_key) {
        Ok(geojson) => CString::new(geojson).unwrap().into_raw(),
        Err(e) => {
            set_last_error(format!("Failed to process DJI log: {}", e));
            std::ptr::null_mut()
        }
    }
}

fn process_dji_log(bytes: &[u8], api_key: &str) -> Result<String, DJILogError> {
    let dji_log = DJILog::from_bytes(bytes.to_vec())?;
    
    let decrypt_method = if dji_log.version >= 13 {
        DecryptMethod::ApiKey(api_key.to_string())
    } else {
        DecryptMethod::None
    };

    let frames = dji_log.frames(decrypt_method)?;
    Ok(DJILog::frames_to_geojson(&frames))
}

#[no_mangle]
pub extern "C" fn get_last_error() -> *mut c_char {
    LAST_ERROR.lock().unwrap().take().map_or(std::ptr::null_mut(), |s| CString::new(s).unwrap().into_raw())
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            drop(CString::from_raw(s));
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum DecryptMethod {
    ApiKey(String),
    Keychains(Vec<Keychain>),
    None,
}

#[derive(Debug)]
pub struct DJILog {
    inner: Vec<u8>,
    prefix: Prefix,
    /// Log format version
    pub version: u8,
    /// Log Details. Contains record summary and general informations
    pub details: Details,
}

impl DJILog {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DJILog, DJILogError> {
        debug!("Parsing DJI log from bytes, length: {}", bytes.len());
        // Decode Prefix
        let mut prefix = Prefix::read(&mut Cursor::new(&bytes))
            .map_err(|e| DJILogError::PrefixParseError(e.to_string()))?;

        let version = prefix.version;

        // Decode Detail
        let detail_offset = prefix.detail_offset() as usize;
        let mut cursor = Cursor::new(pad_with_zeros(&bytes[detail_offset..], 400));

        let details = if version < 13 {
            Details::read_args(&mut cursor, (version,))
                .map_err(|e| DJILogError::DetailsParseError(e.to_string()))?
        } else {
            // Get details from first auxiliary block
            if let Auxiliary::Info(data) = Auxiliary::read(&mut cursor)
                .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
            {
                Details::read_args(&mut Cursor::new(&data.info_data), (version,))
                    .map_err(|e| DJILogError::DetailsParseError(e.to_string()))?
            } else {
                Err(DJILogError::DetailsParseError(
                    "Invalid auxiliary data".into(),
                ))?
            }
        };

        // Try to recover detail offset
        if prefix.records_offset() == 0 && version >= 13 {
            // Skip second auxiliary block
            let _ = Auxiliary::read(&mut cursor)
                .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()));
            prefix.recover_detail_offset(cursor.position() + detail_offset as u64);
        }

        Ok(DJILog {
            inner: bytes,
            prefix,
            version,
            details,
        })
    }

    pub fn keychain_request(&self) -> Result<KeychainRequest, DJILogError> {
        debug!("Creating keychain request, version: {}", self.version);
        println!("Entering keychain_request method");
        let mut keychain_request = KeychainRequest::default();
        
        keychain_request.version = self.version as u16;
        println!("Log version: {}", self.version);
    
        // Extract keychains from KeyStorage Records
        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.records_offset());
    
        let mut keychain: Vec<KeychainCipherText> = Vec::new();
    
        while cursor.position() < self.prefix.records_end_offset(self.inner.len() as u64) {
            let empty_keychain = RefCell::new(HashMap::new());
            let record = match Record::read_args(
                &mut cursor,
                binrw::args! {
                    version: self.version,
                    keychain: &empty_keychain
                },
            ) {
                Ok(record) => record,
                Err(e) => {
                    println!("Error reading record: {:?}", e);
                    break;
                }
            };
    
            match record {
                Record::KeyStorage(data) => {
                    keychain.push(KeychainCipherText {
                        feature_point: data.feature_point,
                        aes_ciphertext: Base64Standard.encode(&data.data),
                    });
                    println!("Found KeyStorage record: {:?}", data.feature_point);
                }
                Record::KeyStorageRecover(_) => {
                    if !keychain.is_empty() {
                        keychain_request.keychains.push(keychain);
                        keychain = Vec::new();
                    }
                    println!("Found KeyStorageRecover record");
                }
                _ => {}
            }
        }
    
        if !keychain.is_empty() {
            keychain_request.keychains.push(keychain);
        }
    
        println!("Keychain request: {:?}", keychain_request);
        Ok(keychain_request)
    }

    pub fn records(&self, decrypt_method: DecryptMethod) -> Result<Vec<Record>, DJILogError> {
        println!("Entering records method");
        if self.version >= 13 && decrypt_method == DecryptMethod::None {
            return Err(DJILogError::RecordParseError(
                "Api Key or keychain is required to parse records".into(),
            ));
        }

        let mut keychains = VecDeque::from(match decrypt_method {
            DecryptMethod::ApiKey(api_key) => {
                println!("Getting keychain request...");
                let request = self.keychain_request()?;
                println!("Fetching keychains...");
                request.fetch(&api_key)?
            },
            DecryptMethod::Keychains(keychains) => keychains,
            DecryptMethod::None => Vec::new(),
        });

        println!("Parsing records...");

        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.records_offset());

        let mut keychain = RefCell::new(keychains.pop_front().unwrap_or(HashMap::new()));

        let mut records = Vec::new();

        while cursor.position() < self.prefix.records_end_offset(self.inner.len() as u64) {
            // decode record
            let record = match Record::read_args(
                &mut cursor,
                binrw::args! {
                    version: self.version,
                    keychain: &keychain
                },
            ) {
                Ok(record) => record,
                Err(_) => break,
            };

            if let Record::KeyStorageRecover(_) = record {
                keychain = RefCell::new(keychains.pop_front().unwrap_or(HashMap::new()));
            }

            records.push(record);
        }

        Ok(records)
    }

    pub fn frames(&self, decrypt_method: DecryptMethod) -> Result<Vec<Frame>, DJILogError> {
        println!("Entering frames method");
        println!("Attempting to get records...");
        let records = self.records(decrypt_method)?;
        println!("Successfully got {} records", records.len());
        if !records.is_empty() {
            println!("First record: {:?}", records[0]);
        }
        println!("Calling records_to_frames...");
        let frames = records_to_frames(records, self.details.clone());
        println!("records_to_frames returned {} frames", frames.len());
        if !frames.is_empty() {
            println!("First frame: {:?}", frames[0]);
        }
        Ok(frames)
    }

    pub fn frames_to_geojson(frames: &[Frame]) -> String {
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
                    }
                })
            }).collect::<Vec<_>>()
        });
        serde_json::to_string(&feature_collection).unwrap()
    }
}


