use base64::{engine::general_purpose::STANDARD as Base64Standard, Engine as _};
use binrw::io::Cursor;
use binrw::BinRead;

use std::error::Error;

mod layout;
mod util;

use crate::layout::auxilliary::{FeaturePoint, InfoData, KeychainData, TypeData, VersionData};
use crate::layout::info::Info;
use crate::layout::prefix::Prefix;
use crate::layout::record_frame::RecordFrame;
use crate::util::decrypt::decrypt;

// Constant definitions for prefix sizes
const OLD_PREFIX_SIZE: usize = 12;
const PREFIX_SIZE: usize = 100;

#[derive(Debug)]
struct ImageData {
    preview: Vec<u8>,
    thumbnail: Vec<u8>,
}

#[derive(Debug)]
pub struct Parser {
    pub version: u8,
    pub info: Info,

    // keychain
    pub keychain_version: u8,
    pub keychain_department: u8,
    pub keychains: Vec<(FeaturePoint, String)>,
}

impl Parser {
    /// Constructs a `Parser` from a byte slice.
    ///
    /// This function parse Info and Keychain data
    pub fn from_bytes(bytes: &[u8]) -> Result<Parser, Box<dyn Error>> {
        // Get Prefix and Offsets
        let prefix = Prefix::read(&mut Cursor::new(bytes))?;
        let version = prefix.format_version;

        let info_offset: usize;
        let records_offset: usize;

        if version < 6 {
            // prefix ; records ; info
            // details_offset points to info section
            info_offset = prefix.detail_offset as usize;
            records_offset = OLD_PREFIX_SIZE;
        } else if version < 12 {
            // prefix ; records ; info
            // details_offset points to info section
            info_offset = prefix.detail_offset as usize;
            records_offset = PREFIX_SIZE;
        } else if version == 12 {
            // prefix ; infos ; records
            // details_offset points to end of file
            info_offset = PREFIX_SIZE;
            records_offset = PREFIX_SIZE + 436 // We manually add info size
        } else {
            // prefix ; infos ; records ; images
            // details_offset points to records section
            info_offset = PREFIX_SIZE;
            records_offset = prefix.detail_offset as usize
        };

        // Get Info and Keychains infos
        let info: Info;
        let mut keychain_version: u8 = 0;
        let mut keychain_department: u8 = 0;

        let mut cursor = Cursor::new(&bytes[info_offset..]);

        if version < 13 {
            info = Info::read_args(&mut cursor, (version,))?;
        } else {
            // Info and keychain version are serialized as auxilliary data structures
            // Unwrap auxilliary info data
            let info_auxilliary_data = TypeData::read(&mut cursor)?;
            // Info data is encrypted
            let decrypted_bytes = util::decrypt::decrypt(
                info_auxilliary_data.type_data,
                &info_auxilliary_data.inner.data,
            );
            let info_data = InfoData::read(&mut Cursor::new(decrypted_bytes))?;
            info = Info::read_args(&mut Cursor::new(&info_data.info.data), (version,))?;

            // Unwrap auxilliary version data
            let version_auxilliary_data = TypeData::read(&mut cursor)?;
            if version_auxilliary_data.type_data == 1 {
                let version_data =
                    VersionData::read(&mut Cursor::new(&version_auxilliary_data.inner.data))?;

                keychain_version = version_data.version as u8;
                keychain_department = version_data.department;
            }
        }

        // Get Keychains
        let keychains = if version >= 13 {
            let mut keychains = Vec::new();
            let mut cursor = Cursor::new(&bytes[records_offset..]);
            for _ in 0..info.record_line_count {
                let frame = RecordFrame::read_args(&mut cursor, (version,))?;
                if frame.record_type == 56 {
                    let decrypted_frame_data = decrypt(frame.record_type, &frame.data);
                    let keychain_data = KeychainData::read(&mut Cursor::new(decrypted_frame_data))?;
                    keychains.push((
                        keychain_data.feature_point,
                        Base64Standard.encode(&keychain_data.inner.data),
                    ));
                }
            }
            keychains
        } else {
            Vec::new()
        };

        Ok(Parser {
            version,
            info,
            keychain_version,
            keychain_department,
            keychains,
        })
    }
}
