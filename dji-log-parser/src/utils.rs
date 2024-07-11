use binrw::{parser, BinResult, Error};

use std::io::SeekFrom;

/// Parses bytes from a reader, checking for the JPEG header `0xFFD8` and reading until the end JPEG marker `0xFFD9` is encountered.
///
/// This function first checks if the initial two bytes match `0xFFD8`. If so, it continuously reads bytes from the given reader and checks
/// for the occurrence of the byte sequence `0xFFD9`. Once this sequence is found, the function stops reading and returns the accumulated bytes
/// (excluding `0xFFD9`). If the initial bytes do not match `0xFFD8`, an error is returned.
///
/// # Parameters
///
/// * `reader`: A mutable reference to an object implementing the `Read` and `Seek` traits.
///             This reader is used to fetch the bytes.
///
/// # Returns
///
/// Returns a `BinResult<Vec<u8>>` which is Ok containing a vector of bytes read until (but
/// not including) `0xFFD9`, or an error (`binrw::Error`) if an issue occurs during reading or
/// if the initial bytes are not `0xFFD8`.
///
/// # Errors
///
/// This function will return an error if there is a problem reading from the reader or if the initial bytes do not match the JPEG header.
#[parser(reader)]
pub fn read_jpeg() -> BinResult<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 2];

    reader.read_exact(&mut temp_buffer)?;

    if temp_buffer != [0xFF, 0xD8] {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid JPEG header",
        )));
    }

    buffer.extend_from_slice(&temp_buffer);

    while reader.read_exact(&mut temp_buffer).is_ok() {
        buffer.push(temp_buffer[0]);

        if temp_buffer == [0xFF, 0xD9] {
            buffer.push(temp_buffer[1]);
            break;
        }

        reader.seek(SeekFrom::Current(-1))?;
    }

    Ok(buffer)
}

/// Seeks through a byte stream to find the start of a JPEG record or an end marker.
///
/// This function reads through bytes from the provided reader two at a time. It looks for
/// a JPEG record, indicated by the byte sequence `0xFF, 0xD8`, or an end marker, indicated
/// by a single `0xFF` byte. If a JPEG record is found, the reader's position is set to the
/// start of this record. If an end marker is found, the reading stops, and the reader's
/// position is at the end marker.
///
/// # Parameters
///
/// * `reader`: A mutable reference to an object implementing the `Read` and `Seek` traits.
///
/// # Returns
///
///  Returns a `BinResult<Vec<u8>>` which is Ok containing a vector of bytes read until either
/// a JPEG record start or an end marker was found. Returns an error (`binrw::Error`) if an
/// issue occurs during reading.
///
/// # Errors
///
/// This function will return an error in the event of a reading failure from the reader.
///
#[parser(reader)]
pub fn seek_to_next_record() -> BinResult<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 2];

    while reader.read_exact(&mut temp_buffer).is_ok() {
        if temp_buffer == [0xFF, 0xD8] {
            reader.seek(SeekFrom::Current(-2))?;
            break;
        }
        if temp_buffer[0] == 0xFF {
            buffer.push(temp_buffer[0]);
            break;
        }
        buffer.push(temp_buffer[0]);
        reader.seek(SeekFrom::Current(-1))?;
    }

    Ok(buffer)
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

/// Ensures that a given slice of bytes has a minimum specified length, padding it with zeros if necessary.
///
/// # Parameters
///
/// - `input`: A slice of bytes (`&[u8]`) to check and possibly extend.
/// - `min_length`: The minimum length desired for the output vector. If `input` is shorter than this,
///    it will be padded with zeros until it reaches `min_length`.
///
/// # Returns
///
/// A `Vec<u8>` that is at least `min_length` bytes long. If the input slice was already at least
/// `min_length` bytes long, this will be a direct copy of `input`. Otherwise, it will be `input`
/// followed by enough zeros to reach `min_length`.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let bytes = b"Hello, world!";
/// let padded_bytes = pad_with_zeros(&bytes[..], 16);
/// assert_eq!(padded_bytes, b"Hello, world!\x00\x00\x00");
/// ```
pub fn pad_with_zeros(input: &[u8], min_length: usize) -> Vec<u8> {
    let current_length = input.len();
    let mut output = Vec::from(input);

    if current_length < min_length {
        let padding = min_length - current_length;
        output.extend(vec![0; padding]);
    }

    output
}

/// Adds a new message to the original message with a separator if the new message is not empty.
///
/// # Arguments
///
/// * `original_message` - The original message to which the new message will be appended.
/// * `message` - The new message to be added. If this message is empty, the original message will remain unchanged.
///
/// # Returns
///
/// A `String` containing the original message followed by the new message separated by "; " if the new message is not empty,
/// otherwise the original message.
///
///
pub fn append_message(original_message: String, message: impl Into<String>) -> String {
    if !original_message.is_empty() {
        format!("{}; {}", original_message, message.into())
    } else {
        message.into()
    }
}
