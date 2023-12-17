use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD as Base64Standard, Engine as _};
use binrw::io::Cursor;
use binrw::BinRead;
use layout::auxiliary::Auxiliary;
use layout::record::FeaturePoint;
use thiserror::Error;

mod decoder;
mod layout;

use crate::layout::info::Info;
use crate::layout::prefix::Prefix;
use crate::layout::record::Record;

#[derive(PartialEq, Debug, Error)]
#[non_exhaustive]
pub enum DJILogError {
    /// Failed to parse Prefix
    #[error("Failed to parse prefix: '{0}'.")]
    PrefixParseError(String),

    /// Failed to parse Info
    #[error("Failed to parse info: '{0}'.")]
    InfoParseError(String),

    /// Failed to parse Auxiliary block
    #[error("Failed to parse auxiliary block: '{0}'.")]
    AuxiliaryParseError(String),

    /// Failed to parse Record
    #[error("Failed to parse record: '{0}'.")]
    RecordParseError(String),

    /// Failed to parse Keychain
    #[error("Failed to parse keychain: '{0}'.")]
    KeychainParseError(String),
}

#[derive(Debug, Default)]
pub struct KeychainInfo {
    version: u16,
    department: u8,
    keychains: HashMap<FeaturePoint, String>,
}

#[derive(Debug)]
pub struct DJILog {
    pub prefix: Prefix,
    pub info: Info,
    pub keychain_info: KeychainInfo,
}

impl DJILog {
    /// Constructs a `Parser` from a byte slice.
    ///
    /// This function parse Prefix, Info and KeychainInfo
    ///
    /// Binary structure:
    ///
    /// v4 -> v6
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │     Records     │                │
    /// ├─────────────────┤<───────────────┘
    /// │      Info       │
    /// └─────────────────┘
    ///
    /// v7 -> v11
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │     Records     │                │
    /// │   (Encrypted)   │                │
    /// ├─────────────────┤<───────────────┘
    /// │      Info       │
    /// └─────────────────┘
    ///
    /// v12
    /// ┌─────────────────┐
    /// │     Prefix      │ detail_offset ─┐
    /// ├─────────────────┤                │
    /// │      Info       │                │
    /// ├─────────────────┤                │
    /// │     Records     │                │
    /// │   (Encrypted)   │                │
    /// └─────────────────┘<───────────────┘
    ///
    /// v13 -> v14
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
    ///
    pub fn from_bytes(bytes: &[u8]) -> Result<DJILog, DJILogError> {
        let mut cursor = Cursor::new(bytes);

        let prefix =
            Prefix::read(&mut cursor).map_err(|e| DJILogError::PrefixParseError(e.to_string()))?;

        let info = Self::parse_info(&mut cursor, &prefix)?;

        let keychain_info = Self::parse_keychain_info(&mut cursor, &prefix)?;

        Ok(DJILog {
            prefix,
            info,
            keychain_info,
        })
    }

    fn parse_info(cursor: &mut Cursor<&[u8]>, prefix: &Prefix) -> Result<Info, DJILogError> {
        cursor.set_position(prefix.info_offset() as u64);

        if prefix.version < 13 {
            Info::read_args(cursor, (prefix.version,))
                .map_err(|e| DJILogError::InfoParseError(e.to_string()))
        } else {
            // Get info from first auxilliary block
            if let Auxiliary::Info(data) = Auxiliary::read(cursor)
                .map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
            {
                Info::read_args(&mut Cursor::new(&data.info_data), (prefix.version,))
                    .map_err(|e| DJILogError::InfoParseError(e.to_string()))
            } else {
                Err(DJILogError::InfoParseError("Invalid auxiliary data".into()))
            }
        }
    }

    fn parse_keychain_info(
        cursor: &mut Cursor<&[u8]>,
        prefix: &Prefix,
    ) -> Result<KeychainInfo, DJILogError> {
        let mut keychain_info = KeychainInfo::default();

        // No keychain
        if prefix.version < 13 {
            return Ok(keychain_info);
        }

        // Get info from first auxilliary block
        let info = Self::parse_info(cursor, prefix)?;

        // Keep cursor position and get version from second auxilliary block
        if let Auxiliary::Version(data) =
            Auxiliary::read(cursor).map_err(|e| DJILogError::AuxiliaryParseError(e.to_string()))?
        {
            keychain_info.version = data.version;
            keychain_info.department = data.department;
        }

        // Extract keychains from KeyStorage Records
        cursor.set_position(prefix.records_offset() as u64);

        for _ in 0..info.record_line_count {
            let record = Record::read_args(cursor, (prefix.version,))
                .map_err(|e| DJILogError::RecordParseError(e.to_string()))?;

            if let Record::KeyStorage(data) = record {
                keychain_info
                    .keychains
                    .insert(data.feature_point, Base64Standard.encode(&data.data));
            }
        }

        Ok(keychain_info)
    }
}
