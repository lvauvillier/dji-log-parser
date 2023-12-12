use binrw::{binread, parser, BinResult};

#[binread]
#[derive(Debug)]
#[br(little, import(format_version: u8))]
pub struct RecordFrame {
    pub record_type: u8,
    #[br(temp, args(format_version <= 12), parse_with = read_u16)]
    length: u16,
    #[br(count = length)]
    pub data: Vec<u8>,
    end: u8,
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
