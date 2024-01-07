//! # DJILog Parser Module
//!
//! This module provides functionality for parsing DJI log files.
//!
//! Binary structure of log files:
//!
//! v4 -> v6
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │     Records     │                │
//! ├─────────────────┤<───────────────┘
//! │      Info       │
//! └─────────────────┘
//! ```
//!
//! v7 -> v11
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │     Records     │                │
//! │   (Encrypted)   │                │
//! ├─────────────────┤<───────────────┘
//! │      Info       │
//! └─────────────────┘
//!```
//!
//! v12
//! ```text
//! ┌─────────────────┐
//! │     Prefix      │ detail_offset ─┐
//! ├─────────────────┤                │
//! │      Info       │                │
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
//! │   (Encrypted)   |                |
//! ├─────────────────┤                │
//! │    Auxiliary    |                |
//! │     Version     |                │
//! ├─────────────────┤<───────────────┘
//! │     Records     │
//! │(Encrypted + AES)|
//! ├─────────────────┤
//! │     Images      │
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
mod keychain;
mod layout;
pub mod record;
mod utils;

pub use crate::keychain::Keychain;
use crate::keychain::{KeychainCipherText, KeychainRequest};
use crate::layout::auxiliary::Auxiliary;
pub use crate::layout::info::Info;
pub use crate::layout::prefix::Prefix;
use crate::record::Record;

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

#[derive(PartialEq)]
pub enum DecryptMethod {
    ApiKey(String),
    Keychains(Vec<Keychain>),
    None,
}

#[derive(Debug)]
pub struct DJILog<'a> {
    inner: &'a [u8],
    pub prefix: Prefix,
    pub info: Info,
}

impl<'a> DJILog<'a> {
    /// Constructs a `DJILog` from a byte slice.
    ///
    /// This function parses the Prefix and Info blocks of the log file,
    /// and handles different versions of the log format.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A byte slice representing the DJI log file.
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
    pub fn from_bytes(bytes: &[u8]) -> Result<DJILog, DJILogError> {
        // Decode Prefix
        let prefix = Prefix::read(&mut Cursor::new(bytes))
            .map_err(|e| DJILogError::PrefixParseError(e.to_string()))?;

        // Decode Infos
        let info_offset = prefix.info_offset() as usize;
        let mut cursor = Cursor::new(&bytes[info_offset..]);

        let info = if prefix.version < 13 {
            Info::read_args(&mut cursor, (prefix.version,))
                .map_err(|e| DJILogError::InfoParseError(e.to_string()))?
        } else {
            // Get info from first auxilliary block
            if let Auxiliary::Info(data) = Auxiliary::read(&mut cursor)
                .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
            {
                Info::read_args(&mut Cursor::new(&data.info_data), (prefix.version,))
                    .map_err(|e| DJILogError::InfoParseError(e.to_string()))?
            } else {
                Err(DJILogError::InfoParseError("Invalid auxiliary data".into()))?
            }
        };

        Ok(DJILog {
            inner: bytes,
            prefix,
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
        if self.prefix.version < 13 {
            return Ok(keychain_request);
        }

        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.info_offset());

        // Skip first auxilliary block
        let _ = Auxiliary::read(&mut cursor)
            .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()));

        // Get version from second auxilliary block
        if let Auxiliary::Version(data) = Auxiliary::read(&mut cursor)
            .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
        {
            keychain_request.version = data.version;
            keychain_request.department = data.department;
        }

        // Extract keychains from KeyStorage Records
        cursor.set_position(self.prefix.records_offset());

        let mut keychain: Vec<KeychainCipherText> = Vec::new();

        let mut i = 0;
        while i < self.info.record_line_count {
            let empty_keychain = RefCell::new(HashMap::new());
            let record = match Record::read_args(
                &mut cursor,
                binrw::args! {
                    version: self.prefix.version,
                    keychain: &empty_keychain
                },
            ) {
                Ok(record) => record,
                Err(e) => {
                    // Recover errors if this is the last iteration of the loop
                    if i == self.info.record_line_count - 1 {
                        break;
                    } else {
                        return Err(DJILogError::RecordParseError(e.to_string()));
                    }
                }
            };

            match record {
                Record::OSD(_) => {
                    i += 1;
                }
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
    /// Retrieves the parsed records from the DJI log.
    ///
    /// This function decodes the records from the log file based on the specified decryption method.
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
        if self.prefix.version >= 13 && decrypt_method == DecryptMethod::None {
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

        let keychain = RefCell::new(keychains.pop_front().unwrap_or(HashMap::new()));

        let mut records = Vec::new();
        let mut i = 0;
        while i < self.info.record_line_count {
            // decode record
            let record = match Record::read_args(
                &mut cursor,
                binrw::args! {
                    version: self.prefix.version,
                    keychain: &keychain
                },
            ) {
                Ok(record) => record,
                Err(e) => {
                    // Recover errors if this is the last iteration of the loop
                    if i == self.info.record_line_count - 1
                        || self.inner.len() as u64 - cursor.position() == 0
                    {
                        break;
                    } else {
                        return Err(DJILogError::RecordParseError(e.to_string()));
                    }
                }
            };

            if let Record::OSD(_) = record {
                i += 1;
            }

            records.push(record);
        }

        Ok(records)
    }
}
