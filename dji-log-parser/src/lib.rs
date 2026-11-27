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
//!    (labeled as the SDK key or App Key).
//!
//! ## Usage
//!
//! ### Initialization
//! Initialize a `DJILog` instance from a byte slice to access version information and metadata:
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! let parser = DJILog::from_bytes(bytes).unwrap();
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
//!
//! ### Access general data
//!
//! General data are not encrypted and can be accessed from the parser for all log versions:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! // Print the log version
//! println!("Version: {:?}", parser.version);
//!
//! // Print the log details section
//! println!("Details: {:?}", parser.details);
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
//!
//! ### Retrieve keychains
//!
//! For logs version 13 and later, keychains must be retrieved from the DJI API to decode the records:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! // Replace `__DJI_API_KEY__` with your actual apiKey
//! let keychains = parser.fetch_keychains("__DJI_API_KEY__").unwrap();
//! # Ok::<(), dji_log_parser::Error>(())
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
//! Each frame holds the state of the aircraft at an OSD record (about 10 per second), in sections:
//! `osd`, `gimbal`, `camera`, `rc`, `battery` (with `battery1` and `battery2` on aircraft with
//! several batteries), `home`, `recover`, `app`, `vision`, `rtk`, `serial`, `firmware` and `custom`.
//!
//! For versions prior to 13:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! let frames = parser.frames(None);
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
//!
//! For version 13 and later:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! # let keychains = parser.fetch_keychains("__DJI_API_KEY__")?;
//! let frames = parser.frames(Some(keychains));
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
//!
//! ### Accessing raw Records
//!
//! Decrypt raw records based on the log file version.
//! For versions prior to 13:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! let records = parser.records(None);
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
//!
//! For version 13 and later:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! # let keychains = parser.fetch_keychains("__DJI_API_KEY__")?;
//! let records = parser.records(Some(keychains));
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
//!
//! ### Reading large logs
//!
//! [`DJILog::records_iter`] and [`DJILog::frames_iter`] read the records and the frames one by
//! one, without keeping them all in memory:
//!
//! ```no_run
//! # use dji_log_parser::DJILog;
//! # let bytes = std::fs::read("DJIFlightRecord.txt")?;
//! # let parser = DJILog::from_bytes(bytes)?;
//! # let keychains = parser.fetch_keychains("__DJI_API_KEY__")?;
//! let max_height = parser
//!     .frames_iter(Some(keychains))?
//!     .map(|frame| frame.osd.height)
//!     .fold(0.0, f32::max);
//! # Ok::<(), dji_log_parser::Error>(())
//! ```
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
use std::borrow::Cow;
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
use frame::{Frame, Frames};
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
    /// ```no_run
    /// use dji_log_parser::DJILog;
    ///
    /// let log_bytes = std::fs::read("DJIFlightRecord.txt")?;
    /// let log = DJILog::from_bytes(log_bytes)?;
    /// # Ok::<(), dji_log_parser::Error>(())
    /// ```
    ///
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DJILog> {
        // Decode Prefix
        let mut prefix = Prefix::read(&mut Cursor::new(&bytes))?;

        let version = prefix.version;

        // Decode Detail
        let detail_offset = prefix.detail_offset() as usize;
        let detail_bytes = bytes.get(detail_offset..).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("detail offset {detail_offset} is beyond end of file"),
            )
        })?;
        // Details parsing expects at least 400 bytes: only copy the end of the file when it is
        // shorter, instead of copying the whole log
        let detail_bytes = if detail_bytes.len() < 400 {
            Cow::Owned(pad_with_zeros(detail_bytes, 400))
        } else {
            Cow::Borrowed(detail_bytes)
        };
        let mut cursor = Cursor::new(detail_bytes);

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

        let records_end = self.prefix.records_end_offset(self.inner.len() as u64);
        loop {
            utils::skip_lone_image_markers(&mut cursor)?;
            if cursor.position() >= records_end {
                break;
            }
            let empty_keychain = &RefCell::new(Keychain::empty());
            let record = match Record::read(
                &mut cursor,
                self.version,
                self.details.product_type,
                empty_keychain,
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
        Ok(self.records_iter(keychains)?.collect())
    }

    /// Iterates over the raw records of the DJI log, decoding each record when it is reached.
    ///
    /// Same records as [`DJILog::records`], without keeping them all in memory.
    ///
    /// # Arguments
    ///
    /// * `keychains` - The keychains of the log, required for encrypted logs (versions >= 13).
    pub fn records_iter(
        &self,
        keychains: Option<Vec<Vec<KeychainFeaturePoint>>>,
    ) -> Result<Records<'_>> {
        if self.version >= 13 && keychains.is_none() {
            return Err(Error::KeychainRequired);
        }

        let mut keychains = VecDeque::from(match keychains {
            Some(keychains) => keychains
                .iter()
                .map(|keychain| Keychain::from_feature_points(keychain))
                .collect::<Result<Vec<_>>>()?,
            None => Vec::new(),
        });

        let mut cursor = Cursor::new(self.inner.as_slice());
        cursor.set_position(self.prefix.records_offset());
        let keychain = RefCell::new(keychains.pop_front().unwrap_or(Keychain::empty()));

        Ok(Records {
            log: self,
            cursor,
            keychains,
            keychain,
            records_end: self.prefix.records_end_offset(self.inner.len() as u64),
            is_done: false,
        })
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
        Ok(self.frames_iter(keychains)?.collect())
    }

    /// Iterates over the normalized frames of the DJI log, building each frame from the records
    /// when it is reached.
    ///
    /// Same frames as [`DJILog::frames`], without keeping all the records and frames in memory.
    ///
    /// # Arguments
    ///
    /// * `keychains` - The keychains of the log, required for encrypted logs (versions >= 13).
    pub fn frames_iter(
        &self,
        keychains: Option<Vec<Vec<KeychainFeaturePoint>>>,
    ) -> Result<Frames<Records<'_>>> {
        Ok(Frames::new(
            self.records_iter(keychains)?,
            self.details.clone(),
        ))
    }
}

/// Iterator over the records of a DJI log, see [`DJILog::records_iter`].
pub struct Records<'a> {
    log: &'a DJILog,
    cursor: Cursor<&'a [u8]>,
    /// Keychains of the next parts of the log, each starting with a `KeyStorageRecover` record
    keychains: VecDeque<Keychain>,
    keychain: RefCell<Keychain>,
    records_end: u64,
    is_done: bool,
}

impl Iterator for Records<'_> {
    type Item = Record;

    fn next(&mut self) -> Option<Record> {
        if self.is_done {
            return None;
        }
        let at_end = utils::skip_lone_image_markers(&mut self.cursor).is_err()
            || self.cursor.position() >= self.records_end;
        if at_end {
            self.is_done = true;
            return None;
        }

        let position = self.cursor.position();
        match Record::read(
            &mut self.cursor,
            self.log.version,
            self.log.details.product_type,
            &self.keychain,
        ) {
            Ok(record) => {
                if let Record::KeyStorageRecover(_) = record {
                    self.keychain =
                        RefCell::new(self.keychains.pop_front().unwrap_or(Keychain::empty()));
                }
                Some(record)
            }
            Err(_) => {
                // A record cut off at the end of a truncated log: its data is kept as invalid
                self.is_done = true;
                let end = (self.records_end as usize).min(self.log.inner.len());
                let rest = self.log.inner.get(position as usize..end)?;
                (!rest.is_empty()).then(|| Record::Invalid(rest.to_vec()))
            }
        }
    }
}
