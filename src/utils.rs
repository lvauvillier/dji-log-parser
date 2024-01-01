use binrw::{parser, BinResult};

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

/// Extracts and aligns a subset of bits from a given byte.
///
/// This function selects bits from the `byte` parameter based on the `mask` parameter.
/// Bits in `byte` that correspond to `1` bits in `mask` are extracted and shifted
/// towards the least significant bit (LSB) position. Bits in positions where `mask` has `0` bits
/// are ignored. The result is a byte containing only the extracted bits, aligned starting
/// from the LSB.
///
/// # Examples
///
/// ```
/// let byte = 0b10101100;
/// let mask = 0b11100000;
/// assert_eq!(sub_byte_field(byte, mask), 0b00000101);
/// ```
///
/// In this example, `sub_byte_field` takes `byte` (0b10101100) and `mask` (0b11100000).
/// The three high-order bits (101) of `byte` are selected and then shifted right by 5 positions
/// to align at the LSB, resulting in 0b00000101.
///
/// # Parameters
///
/// * `byte`: The byte from which to extract bits.
/// * `mask`: A mask to specify which bits to extract. Bits corresponding to `1` bits in the mask
///           are extracted, and `0` bits in the mask are ignored.
///
/// # Returns
///
/// Returns an `u8` byte containing the extracted and LSB-aligned bits from the original `byte`.
///
/// # Notes
///
/// The function assumes that the mask has contiguous `1` bits starting from the most significant bit
/// (MSB). If the mask's `1` bits are not contiguous or do not start from the MSB, the behavior
/// might not be as expected.
pub fn sub_byte_field(byte: u8, mask: u8) -> u8 {
    // First, use "mask" to select the bits that we want, and move them to the low-order part of the byte:
    let mut byte = byte;
    byte &= mask;

    let mut mask = mask;
    while mask != 0x00 && (mask & 0x01) == 0 {
        byte >>= 1;
        mask >>= 1;
    }
    byte
}
