//! # DJILog Parser Module
//!
//! This module provides functionality for parsing DJI log files.
//!
//! ## Encryption in Version 13 and Later
//! Starting with version 13, log records are AES encrypted and require a specific keychain
//! for decryption. This keychain must be obtained from DJI using their API. An apiKey is
//! necessary to access the DJI API.
//!
//! Once keychains are retrieved, they can be stored along with the original log for further
//! offline use.
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
//! let parser = DJILog::from_bytes(bytes).unwrap();
//! ```
//!
//! ### Access general data
//!
//! General data are not encrypted and can be accessed from the parser for all log versions:
//!
//! ```
//! // Print the log version
//! println!("Version: {:?}", parser.version);
//!
//! // Print the log details section
//! println!("Details: {}", parser.details);
//! ```
//!
//! ### Retrieve keychains
//!
//! For logs version 13 and later, keychains must be retrieved from the DJI API to decode the records:
//!
//! ```
//! // Replace `__DJI_API_KEY__` with your actual apiKey
//! let keychains = parser.fetch_keychains("__DJI_API_KEY__").unwrap();
//! ```
//!
//! Keychains can be retrieved once, serialized, and stored along with the log file for future offline use.
//!
//! ### Accessing Frames
//!
//! Decrypt frames based on the log file version.
//!
//! A `Frame` is a standardized representation of log data, normalized across different log versions.
//! It provides a consistent and easy-to-use format for analyzing and processing DJI log information.
//!
//! For versions prior to 13:
//!
//! ```
//! let frames = parser.frames(None);
//! ```
//!
//! For version 13 and later:
//!
//! ```
//! let frames = parser.frames(Some(keychains));
//! ```
//!
//! ### Accessing raw Records
//!
//! Decrypt raw records based on the log file version.
//! For versions prior to 13:
//!
//! ```
//! let records = parser.records(None);
//! ```
//!
//! For version 13 and later:
//!
//! ```
//! let records = parser.records(Some(keychains));
//! ```
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
use std::collections::VecDeque;

mod decoder;
mod error;
pub mod frame;
pub mod keychain;
pub mod layout;
pub mod record;
mod utils;

pub use error::{Error, Result};
use frame::{records_to_frames, Frame};
use keychain::{EncodedKeychainFeaturePoint, Keychain, KeychainFeaturePoint, KeychainsRequest};
use layout::auxiliary::{Auxiliary, Department};
use layout::details::Details;
use layout::prefix::Prefix;
use record::Record;

use crate::utils::pad_with_zeros;

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
    /// Constructs a `DJILog` from an array of bytes.
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
    /// This function returns `Result<DJILog>`.
    /// On success, it returns the `DJILog` instance.
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
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DJILog> {
        // Decode Prefix
        let mut prefix = Prefix::read(&mut Cursor::new(&bytes))?;

        let version = prefix.version;

        // Decode Detail
        let detail_offset = prefix.detail_offset() as usize;
        let mut cursor = Cursor::new(pad_with_zeros(&bytes[detail_offset..], 400));

        let details = if version < 13 {
            Details::read_args(&mut cursor, (version,))?
        } else {
            // Get details from first auxiliary block
            if let Auxiliary::Info(data) = Auxiliary::read(&mut cursor)? {
                Details::read_args(&mut Cursor::new(&data.info_data), (version,))?
            } else {
                return Err(Error::MissingAuxilliaryData("Info".into()));
            }
        };

        // Try to recover detail offset
        if prefix.records_offset() == 0 && version >= 13 {
            // Skip second auxiliary block
            let _ = Auxiliary::read(&mut cursor)?;
            prefix.recover_detail_offset(cursor.position() + detail_offset as u64);
        }

        Ok(DJILog {
            inner: bytes,
            prefix,
            version,
            details,
        })
    }

    /// Creates a `KeychainsRequest` object by parsing `KeyStorage` records.
    ///
    /// This function is used to build a request body for manually retrieving the keychain from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    /// For earlier versions, this function returns a default `KeychainsRequest`.
    ///
    /// # Returns
    ///
    /// Returns a `Result<KeychainsRequest>`. On success, it provides a `KeychainsRequest`
    /// instance, which contains the necessary information to fetch keychains from the DJI API.
    ///
    pub fn keychains_request(&self) -> Result<KeychainsRequest> {
        self.keychains_request_with_custom_params(None, None)
    }

    /// Creates a `KeychainsRequest` object by parsing `KeyStorage` records with manually specified params.
    ///
    /// This function is used to build a request body for manually retrieving the keychain from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    /// For earlier versions, this function returns a default `KeychainsRequest`.
    ///
    /// # Arguments
    ///
    /// * `department` - An optional `Department` to manually set in the request. If `None`, the department
    ///   will be determined from the log file.
    /// * `version` - An optional version number to manually set in the request. If `None`, the version
    ///   will be determined from the log file.
    ///
    /// # Returns
    ///
    /// Returns a `Result<KeychainsRequest>`. On success, it provides a `KeychainsRequest`
    /// instance, which contains the necessary information to fetch keychains from the DJI API.
    ///
    pub fn keychains_request_with_custom_params(
        &self,
        department: Option<Department>,
        version: Option<u16>,
    ) -> Result<KeychainsRequest> {
        let mut keychain_request = KeychainsRequest::default();

        // No keychain
        if self.version < 13 {
            return Ok(keychain_request);
        }

        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.detail_offset());

        // Skip first auxiliary block
        let _ = Auxiliary::read(&mut cursor)?;

        // Get version from second auxilliary block
        if let Auxiliary::Version(data) = Auxiliary::read(&mut cursor)? {
            // Use provided version or determine from log
            keychain_request.version = version.unwrap_or(data.version);
            // Use provided department or determine from log
            keychain_request.department = match department {
                Some(dept) => dept.into(),
                None => match data.department {
                    Department::Unknown(_) => Department::DJIFly.into(),
                    _ => data.department.into(),
                },
            };
        } else {
            return Err(Error::MissingAuxilliaryData("Version".into()));
        }

        // Extract keychains from KeyStorage Records
        cursor.set_position(self.prefix.records_offset());

        let mut keychain: Vec<EncodedKeychainFeaturePoint> = Vec::new();

        while cursor.position() < self.prefix.records_end_offset(self.inner.len() as u64) {
            let empty_keychain = &RefCell::new(Keychain::empty());
            let record = match Record::read_args(
                &mut cursor,
                binrw::args! {
                    version: self.version,
                    keychain: empty_keychain
                },
            ) {
                Ok(record) => record,
                Err(_) => break,
            };

            match record {
                Record::KeyStorage(data) => {
                    // add EncodedKeychainFeaturePoint to current keychain
                    keychain.push(EncodedKeychainFeaturePoint {
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

    /// Fetches keychains using the provided API key.
    ///
    /// This function first creates a `KeychainRequest` using the `keychain_request()` method,
    /// then uses that request to fetch the actual keychains from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A string slice that holds the API key for authentication with the DJI API.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Vec<KeychainFeaturePoint>>>`. On success, it provides a vector of vectors,
    /// where each inner vector represents a keychain.
    ///
    #[cfg(not(target_arch = "wasm32"))]
    pub fn fetch_keychains(&self, api_key: &str) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        if self.version >= 13 {
            self.keychains_request()?.fetch(api_key, None)
        } else {
            Ok(Vec::new())
        }
    }

    /// Fetches keychains asynchronously using the provided API key.
    /// Available on wasm and native behind the `native-async` feature.
    ///
    /// This function first creates a `KeychainRequest` using the `keychain_request()` method,
    /// then uses that request to asynchronously fetch the actual keychains from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A string slice that holds the API key for authentication with the DJI API.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Vec<KeychainFeaturePoint>>>`. On success, it provides a vector of vectors,
    /// where each inner vector represents a keychain.
    ///
    #[cfg(any(target_arch = "wasm32", feature = "native-async"))]
    pub async fn fetch_keychains_async(
        &self,
        api_key: &str,
    ) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        if self.version >= 13 {
            self.keychains_request()?.fetch_async(api_key, None).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieves the parsed raw records from the DJI log.
    ///
    /// This function decodes the raw records from the log file
    ///
    /// # Arguments
    ///
    /// * `keychains` - An optional vector of vectors containing `KeychainFeaturePoint` instances. This parameter
    ///   is used for decryption when working with encrypted logs (versions >= 13). If `None` is provided,
    ///   the function will attempt to process the log without decryption.
    ///
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Record>>`. On success, it provides a vector of `Record`
    /// instances representing the parsed log records.
    ///
    pub fn records(
        &self,
        keychains: Option<Vec<Vec<KeychainFeaturePoint>>>,
    ) -> Result<Vec<Record>> {
        if self.version >= 13 && keychains.is_none() {
            return Err(Error::KeychainRequired);
        }

        let mut keychains = VecDeque::from(match keychains {
            Some(keychains) => keychains
                .iter()
                .map(Keychain::from_feature_points)
                .collect(),
            None => Vec::new(),
        });

        let mut cursor = Cursor::new(&self.inner);
        cursor.set_position(self.prefix.records_offset());

        let mut keychain = RefCell::new(keychains.pop_front().unwrap_or(Keychain::empty()));

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
                keychain = RefCell::new(keychains.pop_front().unwrap_or(Keychain::empty()));
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
    /// * `keychains` - An optional vector of vectors containing `KeychainFeaturePoint` instances. This parameter
    ///   is used for decryption when working with encrypted logs (versions >= 13). If `None` is provided,
    ///   the function will attempt to process the log without decryption.
    ///
    ///
    /// # Returns
    ///
    /// Returns a `Result<Vec<Frame>>`. On success, it provides a vector of `Frame`
    /// instances representing the normalized log data.
    ///
    /// # Note
    ///
    /// This method consumes and processes the raw records to create frames. It's generally preferred
    /// over using raw records directly, as frames provide a consistent format across different log
    /// versions, simplifying data analysis and interpretation.
    ///
    pub fn frames(&self, keychains: Option<Vec<Vec<KeychainFeaturePoint>>>) -> Result<Vec<Frame>> {
        let records = self.records(keychains)?;
        Ok(records_to_frames(records, self.details.clone()))
    }
}
