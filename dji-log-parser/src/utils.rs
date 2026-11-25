use binrw::{parser, BinResult, Error};

use std::io::{Read, SeekFrom};

/// Marker written before images: before the images at the end of v13+ logs, and before the
/// images embedded in the records of older logs.
const IMAGE_MARKER: [u8; 4] = 12345i32.to_le_bytes();

fn invalid_jpeg(message: &str) -> Error {
    Error::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message,
    ))
}

fn read_byte<R: Read>(reader: &mut R) -> BinResult<u8> {
    let mut byte = [0];
    reader.read_exact(&mut byte)?;
    Ok(byte[0])
}

/// Reads a JPEG image, optionally preceded by the image marker.
///
/// The image is read segment by segment, skipping each segment by its length, up to the end of
/// image marker `0xFFD9`: the EXIF data can contain a thumbnail, with its own end of image marker.
#[parser(reader)]
pub fn read_jpeg() -> BinResult<Vec<u8>> {
    let start = reader.stream_position()?;
    let mut marker = [0; 4];
    if reader.read_exact(&mut marker).is_err() || marker != IMAGE_MARKER {
        reader.seek(SeekFrom::Start(start))?;
    }

    let mut image = vec![read_byte(reader)?, read_byte(reader)?];
    if image != [0xFF, 0xD8] {
        return Err(invalid_jpeg("invalid JPEG header"));
    }

    loop {
        if read_byte(reader)? != 0xFF {
            return Err(invalid_jpeg("invalid JPEG marker"));
        }
        image.push(0xFF);
        // Markers can be preceded by any number of 0xFF fill bytes
        let mut code = read_byte(reader)?;
        while code == 0xFF {
            image.push(code);
            code = read_byte(reader)?;
        }
        image.push(code);

        match code {
            // End of image
            0xD9 => return Ok(image),
            // Markers without segment
            0x01 | 0xD0..=0xD7 => continue,
            _ => {}
        }

        let length = [read_byte(reader)?, read_byte(reader)?];
        image.extend_from_slice(&length);
        let mut segment = vec![0; usize::from(u16::from_be_bytes(length)).saturating_sub(2)];
        reader.read_exact(&mut segment)?;
        image.extend_from_slice(&segment);

        // Start of scan: the entropy-coded data runs up to the next marker. In this data, 0xFF is
        // followed by 0x00 (stuffing) or a restart marker.
        if code == 0xDA {
            loop {
                let byte = read_byte(reader)?;
                if byte != 0xFF {
                    image.push(byte);
                    continue;
                }
                let next = read_byte(reader)?;
                if next == 0x00 || (0xD0..=0xD7).contains(&next) {
                    image.extend_from_slice(&[byte, next]);
                    continue;
                }
                reader.seek(SeekFrom::Current(-2))?;
                break;
            }
        }
    }
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
            // The second byte is the start of the next record
            reader.seek(SeekFrom::Current(-1))?;
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
/// ```ignore
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
///   are extracted, and `0` bits in the mask are ignored.
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
///   it will be padded with zeros until it reaches `min_length`.
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
/// ```ignore
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
