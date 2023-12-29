use std::cell::RefCell;

use binrw::io::NoSeek;
use binrw::{binread, parser, BinResult};

use crate::decoder::record_decoder;
use crate::layout::feature_point::FeaturePoint;
use crate::Keychain;

#[binread]
#[derive(Debug)]
#[br(little, import(version: u8, keychain: &RefCell<Keychain>))]
pub enum Record {
    #[br(magic = 1u8)]
    OSDFlightRecordDataType(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_0,
            count = if version <= 12 {
                self_0
            } else {
                self_0 - 2
            },
            map_stream = |reader| NoSeek::new(record_decoder(reader, 1, version, keychain, self_0)) )]
        Vec<u8>,
        #[br(temp)] u8, // end byte
    ),
    #[br(magic = 56u8)]
    KeyStorage(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_0, map_stream = |reader| NoSeek::new(record_decoder(reader, 56, version, keychain, self_0)))]
         KeyStorage,
        #[br(temp)] u8, // end byte
    ),
    #[br(magic = 50u8)]
    FlightRecordRecover(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(count = self_0)] Vec<u8>,
        #[br(temp)] u8, // end byte
    ),
    Unknown(
        u8, // record_type
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_1,
            count = if version <= 12 {
                self_1
            } else {
                self_1 - 2
            },
            map_stream = |reader| NoSeek::new(record_decoder(reader, self_0, version, keychain, self_1)))]
        Vec<u8>,
        #[br(temp)] u8, // end byte
    ),
}

/// Reads a 16-bit unsigned integer from a given reader.
///
/// This function attempts to read 2 bytes from the specified reader and convert them into a `u16`. It can adjust its behavior based on the `from_u8` flag.
///
/// # Parameters
/// - `from_u8`: A boolean flag indicating how to read the bytes.
///   - If `true`, only one byte is read from the reader, and it's treated as the lower part of the `u16`.
///   - If `false`, two bytes are read and directly converted to `u16`.
///
/// # Returns
/// This function returns a `BinResult<u16>`. On successful read and conversion, it returns `Ok(u16)`.
/// If there is any error in reading from the reader, it returns an error encapsulated in the `BinResult`.
///
/// # Errors
/// This function will return an error if the reader fails to provide the necessary number of bytes (1 or 2, based on `from_u8`).
/// ```
#[parser(reader)]
pub fn read_u16(from_u8: bool) -> BinResult<u16> {
    let mut bytes = [0; 2];
    reader.read_exact(&mut bytes[if from_u8 { 0..=0 } else { 0..=1 }])?;
    Ok(u16::from_le_bytes(bytes))
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct KeyStorage {
    pub feature_point: FeaturePoint,
    #[br(temp)]
    data_length: u16,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}
