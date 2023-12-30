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

pub use crate::keychain::Keychain;
use crate::keychain::{KeychainCipherText, KeychainRequest};
use crate::layout::auxiliary::Auxiliary;
use crate::layout::info::Info;
use crate::layout::prefix::Prefix;
use crate::layout::record::Record;

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
    /// Constructs a `Parser` from a byte slice.
    ///
    /// This function parse Prefix, Info and KeychainInfo
    ///
    /// Binary structure:
    ///
    /// v4 -> v6
    /// ```text
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │     Records     │                │
    /// ├─────────────────┤<───────────────┘
    /// │      Info       │
    /// └─────────────────┘
    /// ```
    ///
    /// v7 -> v11
    /// ```text
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │     Records     │                │
    /// │   (Encrypted)   │                │
    /// ├─────────────────┤<───────────────┘
    /// │      Info       │
    /// └─────────────────┘
    ///```
    ///
    /// v12
    /// ```text
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │      Info       │                │
    /// ├─────────────────┤                │
    /// │     Records     │                │
    /// │   (Encrypted)   │                │
    /// └─────────────────┘<───────────────┘
    ///```
    ///
    /// v13 -> v14
    /// ```text
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │ Auxiliary Info  |                |
    /// │   (Encrypted)   |                |
    /// ├─────────────────┤                │
    /// │    Auxiliary    |                |
    /// │     Version     |                │
    /// ├─────────────────┤<───────────────┘
    /// │     Records     │
    /// │(Encrypted + AES)|
    /// ├─────────────────┤
    /// │     Images      │
    /// └─────────────────┘
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<DJILog, DJILogError> {
        // Create a cursor
        let mut cursor = Cursor::new(bytes);

        // Decode Prefix
        let prefix =
            Prefix::read(&mut cursor).map_err(|e| DJILogError::PrefixParseError(e.to_string()))?;

        // Decode Infos
        cursor.set_position(prefix.info_offset() as u64);

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

    /// Create a KeychainRequest object by parsing KeyStorage records
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
            let record = Record::read_args(
                &mut cursor,
                (self.prefix.version, &RefCell::new(HashMap::new())),
            )
            .map_err(|e| DJILogError::RecordParseError(e.to_string()))?;

            match record {
                Record::OSDFlightRecordDataType(_) => {
                    i += 1;
                }
                Record::KeyStorage(data) => {
                    // add KeychainCipherText to current keychain
                    keychain.push(KeychainCipherText {
                        feature_point: data.feature_point,
                        aes_ciphertext: Base64Standard.encode(&data.data),
                    });
                }
                Record::FlightRecordRecover(_) => {
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

        let mut keychain = RefCell::new(keychains.pop_front().unwrap_or(HashMap::new()));

        let mut i = 0;
        while i < self.info.record_line_count {
            let record = Record::read_args(&mut cursor, (self.prefix.version, &keychain))
                .map_err(|e| DJILogError::RecordParseError(e.to_string()))?;

            match record {
                Record::OSDFlightRecordDataType(data) => {
                    i += 1;
                    println!("OSDFlightRecordDataType {:?}", data);
                }
                Record::FlightRecordRecover(_) => {
                    keychain = RefCell::new(keychains.pop_front().unwrap_or(HashMap::new()));
                }
                Record::Unknown(record_type, data) => {
                    println!("Unknown ({}): {:?}", record_type, data);
                }
                _ => {}
            }
        }

        Ok(Vec::new())
    }
}
