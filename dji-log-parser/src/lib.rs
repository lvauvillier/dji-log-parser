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

mod decoder;
pub mod frame;
pub mod keychain;
pub mod layout;
pub mod record;
mod utils;

use frame::{records_to_frames, Frame};
use keychain::{Keychain, KeychainCipherText, KeychainRequest};
use layout::auxiliary::Auxiliary;
use layout::details::Details;
use layout::prefix::Prefix;
use record::Record;

use crate::utils::pad_with_zeros;

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
        let mut keychain_request = KeychainRequest::default();

        // No keychain
        if self.version < 13 {
            return Ok(keychain_request);
        }

        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.detail_offset());

        // Skip first auxiliary block
        let _ = Auxiliary::read(&mut cursor)
            .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()));

        // Get version from second auxilliary block
        if let Auxiliary::Version(data) = Auxiliary::read(&mut cursor)
            .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
        {
            keychain_request.version = data.version;
            keychain_request.department = data.department.into();
        }

        // Extract keychains from KeyStorage Records
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
                Err(_) => break,
            };

            match record {
                Record::KeyStorage(data) => {
                    // add KeychainCipherText to current keychain
                    keychain.push(KeychainCipherText {
                        feature_point: data.feature_point,
                        aes_ciphertext: Base64Standard.encode(&data.data),
                    });
                }
                Record::KeyStorageRecover(_) => {
                    // start a new keychain
                    keychain_request.keychains.push(keychain);
                    keychain = Vec::new();
                }
                _ => {}
            }
        }

        keychain_request.keychains.push(keychain);

        Ok(keychain_request)
    }

    pub fn records(&self, decrypt_method: DecryptMethod) -> Result<Vec<Record>, DJILogError> {
        if self.version >= 13 && decrypt_method == DecryptMethod::None {
            return Err(DJILogError::RecordParseError(
                "Api Key or keychain is required to parse records".into(),
            ));
        }

        let mut keychains = VecDeque::from(match decrypt_method {
            DecryptMethod::ApiKey(api_key) => self.keychain_request()?.fetch(&api_key)?,
            DecryptMethod::Keychains(keychains) => keychains,
            DecryptMethod::None => Vec::new(),
        });

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
        let records = self.records(decrypt_method)?;
        println!("Number of records: {}", records.len());
        let frames = records_to_frames(records, self.details.clone());
        println!("Number of frames: {}", frames.len());
        Ok(frames)
    }
}

use std::os::raw::c_char;
use std::ffi::{CStr, CString};
use std::fs;
use std::path::Path;
use serde_json;

static mut LAST_ERROR: Option<String> = None;

#[no_mangle]
pub extern "C" fn parse_dji_log(filename: *const c_char, api_key: *const c_char) -> bool {
    let c_str = unsafe { CStr::from_ptr(filename) };
    let filename = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid filename");
            return false;
        }
    };
    
    let c_str_key = unsafe { CStr::from_ptr(api_key) };
    let api_key = match c_str_key.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error("Invalid API key");
            return false;
        }
    };

    println!("Parsing file: {}", filename);
    println!("Using API key: {}", api_key);

    // Read the file
    let bytes = match fs::read(filename) {
        Ok(b) => b,
        Err(e) => {
            set_last_error(&format!("Failed to read file: {}", e));
            return false;
        }
    };

    println!("File size: {} bytes", bytes.len());

    // Parse the log
    let parser = match DJILog::from_bytes(bytes) {
        Ok(p) => p,
        Err(e) => {
            set_last_error(&format!("Failed to parse log: {}", e));
            return false;
        }
    };

    println!("Log version: {}", parser.version);

    // Get frames
    let frames = match parser.frames(DecryptMethod::ApiKey(api_key.to_string())) {
        Ok(f) => f,
        Err(e) => {
            set_last_error(&format!("Failed to get frames: {}. Error details: {:?}", e, e));
            return false;
        }
    };

    println!("Number of frames: {}", frames.len());
    if !frames.is_empty() {
        println!("First frame: {:?}", frames[0]);
    }

    // Convert frames to GeoJSON
    let geojson = frames_to_geojson(&frames);

    if geojson.is_empty() {
        set_last_error("GeoJSON conversion resulted in empty string");
        return false;
    }

    // Write GeoJSON to a file
    let output_path = Path::new(filename).with_extension("json");
    if let Err(e) = fs::write(&output_path, &geojson) {
        set_last_error(&format!("Failed to write GeoJSON: {}", e));
        return false;
    }

    println!("GeoJSON written to: {:?}", output_path);

    true
}

#[no_mangle]
pub extern "C" fn get_last_error() -> *mut c_char {
    let error = unsafe {
        LAST_ERROR.take().unwrap_or_else(|| "No error".to_string())
    };
    CString::new(error).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    unsafe {
        if s.is_null() { return }
        drop(CString::from_raw(s));
    };
}

#[no_mangle]
pub extern "C" fn get_geojson_file_path(filename: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(filename) };
    let filename = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("").unwrap().into_raw(),
    };
    
    let path = Path::new(filename).with_extension("json");
    CString::new(path.to_str().unwrap()).unwrap().into_raw()
}

fn set_last_error(error: &str) {
    unsafe {
        LAST_ERROR = Some(error.to_string());
    }
}

fn frames_to_geojson(frames: &[Frame]) -> String {
    println!("Converting {} frames to GeoJSON", frames.len());
    if frames.is_empty() {
        println!("No frames to convert!");
        return "{}".to_string();
    }
    println!("First frame: {:?}", frames[0]);
    let result = serde_json::to_string(&frames);
    match result {
        Ok(json) => {
            println!("Successfully converted frames to JSON. First 100 characters: {}", &json[..std::cmp::min(100, json.len())]);
            json
        },
        Err(e) => {
            println!("Error converting frames to JSON: {}", e);
            println!("Attempting to serialize individual frames:");
            for (i, frame) in frames.iter().enumerate() {
                match serde_json::to_string(frame) {
                    Ok(_) => println!("Frame {} serialized successfully", i),
                    Err(e) => println!("Error serializing frame {}: {}", i, e),
                }
            }
            "{}".to_string()
        }
    }
}