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
//! println!("Info: {:?}", parser.info);
//! ```
//!
//! ### Accessing Records
//! Decrypt records based on the log file version.
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
//!
//! ### Advanced: Manual Keychain Retrieval
//! For scenarios like caching, offline use, or custom server communication, the library
//! exposes the internal keychain retrieval process:
//! ```rust
//! let keychain_request = parser.keychain_request().unwrap();
//! let keychains = keychain_request.fetch("__DJI_API_KEY__").unwrap();
//! let records = parser.records(DecryptMethod::Keychains(keychains));
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
//! │      Info       │ detail_length
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
//! │      Info       │ detail_length
//! └─────────────────┘
//!```
//!
//! v12
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │      Info       │ detail_length  │
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
//! │ Auxiliary Info  |  detail_length |
//! │   (Encrypted)   |                |
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
use layout::info::Info;
use layout::prefix::Prefix;
use record::Record;

use crate::utils::pad_with_zeros;

#[derive(PartialEq, Debug, Error)]
#[non_exhaustive]
pub enum DJILogError {
    #[error("Failed to parse prefix: {0}")]
    PrefixParseError(String),

    #[error("Failed to parse info: {0}")]
    InfoParseError(String),

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
    /// Log info. Contains record summary and general informations
    pub info: Info,
}

impl DJILog {
    /// Constructs a `DJILog` from an arry of bytes.
    ///
    /// This function parses the Prefix and Info blocks of the log file,
    /// and handles different versions of the log format.
    ///
    /// # Arguments
    ///
    /// * `bytes` - An array of bytes representing the DJI log file.
    ///
    /// # Returns
    ///
    /// This function returns `Result<DJILog, DJILogError>`.
    /// On success, it returns the `DJILog` instance. On failure, it returns
    /// a `DJILogError` indicating the type of error encountered.
    ///
    /// # Examples
    ///
    /// ```
    /// use djilog_parser::DJILog;
    ///
    /// let log_bytes = include_bytes!("path/to/log/file");
    /// let log = DJILog::from_bytes(log_bytes).unwrap();
    /// ```
    ///
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DJILog, DJILogError> {
        // Decode Prefix
        let mut prefix = Prefix::read(&mut Cursor::new(&bytes))
            .map_err(|e| DJILogError::PrefixParseError(e.to_string()))?;

        let version = prefix.version;

        // Decode Infos
        let info_offset = prefix.info_offset() as usize;
        let mut cursor = Cursor::new(pad_with_zeros(&bytes[info_offset..], 400));

        let info = if version < 13 {
            Info::read_args(&mut cursor, (version,))
                .map_err(|e| DJILogError::InfoParseError(e.to_string()))?
        } else {
            // Get info from first auxiliary block
            if let Auxiliary::Info(data) = Auxiliary::read(&mut cursor)
                .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
            {
                Info::read_args(&mut Cursor::new(&data.info_data), (version,))
                    .map_err(|e| DJILogError::InfoParseError(e.to_string()))?
            } else {
                Err(DJILogError::InfoParseError("Invalid auxiliary data".into()))?
            }
        };

        // Try to recover detail offset
        if prefix.records_offset() == 0 && version >= 13 {
            // Skip second auxiliary block
            let _ = Auxiliary::read(&mut cursor)
                .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()));
            prefix.recover_detail_offset(cursor.position() + info_offset as u64);
        }

        Ok(DJILog {
            inner: bytes,
            prefix,
            version,
            info,
        })
    }

    /// Creates a `KeychainRequest` object by parsing `KeyStorage` records.
    ///
    /// This function is used to build a request body for manually retrieving the keychain from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    /// For earlier versions, this function returns a default `KeychainRequest`.
    ///
    /// # Returns
    ///
    /// Returns a `Result<KeychainRequest, DJILogError>`. On success, it provides a `KeychainRequest`
    /// instance, which contains the necessary information to fetch keychains from the DJI API.
    /// On failure, it returns a `DJILogError` indicating the type of error encountered during parsing.
    ///
    pub fn keychain_request(&self) -> Result<KeychainRequest, DJILogError> {
        let mut keychain_request = KeychainRequest::default();

        // No keychain
        if self.version < 13 {
            return Ok(keychain_request);
        }

        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.info_offset());

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
                Record::Recover(_) => {
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

    /// Retrieves the parsed raw records from the DJI log.
    ///
    /// This function decodes the raw records from the log file based on the specified decryption method.
    /// For log versions less than 13, `DecryptMethod::None` should be used as there is no encryption.
    /// For versions 13 and above, records are encrypted and require a decryption method:
    /// - `DecryptMethod::Keychains` if you want to manually provide the keychains,
    /// - `DecryptMethod::ApiKey` if you have an API key to decrypt the records.
    ///
    /// # Arguments
    ///
    /// * `decrypt_method` - The method used to decrypt the log records. This should be chosen based on the log version and available decryption keys.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Record>, DJILogError>`. On success, it provides a vector of `Record`
    /// instances representing the parsed log records. On failure, it returns a `DJILogError` indicating
    /// the type of error encountered during record parsing.
    ///
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

            if let Record::Recover(_) = record {
                keychain = RefCell::new(keychains.pop_front().unwrap_or(HashMap::new()));
            }

            records.push(record);
        }

        Ok(records)
    }

    /// Retrieves the normalized frames from the DJI log.
    ///
    /// This function processes the raw records from the log file and converts them into standardized
    /// frames. Frames are a more user-friendly representation of the log data, normalized across all
    /// log versions for easier use and analysis.
    ///
    /// The function first decodes the raw records based on the specified decryption method, then
    /// converts these records into frames. This normalization process makes it easier to work with
    /// log data from different DJI log versions.
    ///
    /// # Arguments
    ///
    /// * `decrypt_method` - The method used to decrypt the log records. This should be chosen based
    ///   on the log version and available decryption keys:
    ///   - For log versions < 13, use `DecryptMethod::None` (no encryption).
    ///   - For log versions >= 13, use either:
    ///     - `DecryptMethod::Keychains` if manually providing keychains, or
    ///     - `DecryptMethod::ApiKey` if using an API key for decryption.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Frame>, DJILogError>`. On success, it provides a vector of `Frame`
    /// instances representing the normalized log data. On failure, it returns a `DJILogError`
    /// indicating the type of error encountered during frame processing.
    ///
    /// # Note
    ///
    /// This method consumes and processes the raw records to create frames. It's generally preferred
    /// over using raw records directly, as frames provide a consistent format across different log
    /// versions, simplifying data analysis and interpretation.
    ///
    pub fn frames(&self, decrypt_method: DecryptMethod) -> Result<Vec<Frame>, DJILogError> {
        let records = self.records(decrypt_method)?;
        Ok(records_to_frames(records))
    }
}
