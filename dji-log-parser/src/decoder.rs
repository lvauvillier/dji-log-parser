use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockSizeUser, KeyIvInit};
use aes::Aes256;
use crc64::crc64;
use std::cell::RefCell;
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

use crate::keychain::{FeaturePoint, Keychain};

type Aes256CbcDec = cbc::Decryptor<Aes256>;

pub trait SeekRead: Seek + Read {}
impl<T> SeekRead for T where T: Seek + Read {}

/// Constructs a reader based on the given record type, version, and keychain.
///
/// # Arguments
///
/// * `reader` - A reader that implements the `Read` trait.
/// * `record_type` - The type of record to be read.
/// * `version` - The prefix version.
/// * `keychain` - A reference to a keychain for decryption. We use here a RefCell here to allow updates and satisfy the Fn immutability constraint
/// * `size` - The size of the data to be read.
///
/// # Returns
///
/// A boxed reader implementing `Read + Seek`. binrw's `map_stream` cannot propagate errors,
/// so if the decoder cannot be built (truncated or corrupted data), the returned reader
/// fails on first access and binrw reports it as a regular parse error.
pub fn record_decoder<'a, R>(
    reader: R,
    record_type: u8,
    version: u8,
    keychain: &RefCell<Keychain>,
    size: u16,
) -> Box<dyn SeekRead + 'a>
where
    R: Read + Seek + 'a,
{
    try_record_decoder(reader, record_type, version, keychain, size)
        .unwrap_or_else(|error| Box::new(FailedDecoder::from(error)))
}

/// Xor decoder for use in binrw's `map_stream`, with the same error fallback as [`record_decoder`].
pub fn xor_decoder<'a, R>(reader: R, record_type: u8) -> Box<dyn SeekRead + 'a>
where
    R: Read + Seek + 'a,
{
    match XorDecoder::new(reader, record_type) {
        Ok(decoder) => Box::new(decoder),
        Err(error) => Box::new(FailedDecoder::from(error)),
    }
}

fn try_record_decoder<'a, R>(
    reader: R,
    record_type: u8,
    version: u8,
    keychain: &RefCell<Keychain>,
    size: u16,
) -> Result<Box<dyn SeekRead + 'a>>
where
    R: Read + Seek + 'a,
{
    match version {
        // Raw
        0..=6 => Ok(Box::new(reader)),
        // Xor
        7..=12 => Ok(Box::new(XorDecoder::new(reader, record_type)?)),
        // Xor + AES
        _ => {
            let feature_point = FeaturePoint::from_record_type(record_type, version);
            if feature_point == FeaturePoint::PlaintextFeature {
                return Ok(Box::new(XorDecoder::new(reader, record_type)?));
            }

            let pair = keychain
                .borrow()
                .get(&feature_point)
                .map(|value| (value.0.clone(), value.1.clone()));

            let Some((iv, key)) = pair else {
                return Ok(Box::new(XorDecoder::new(reader, record_type)?));
            };

            // firstChar and lastChar are not part of the content
            let content_size = size
                .checked_sub(2)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "record too short"))?;

            let aes_reader = AesDecoder::new(
                XorDecoder::new(reader, record_type)?,
                &iv,
                &key,
                content_size,
                size,
            )?;

            // Update keychain with next iv
            keychain
                .borrow_mut()
                .insert(feature_point, (aes_reader.next_iv.clone(), key));

            Ok(Box::new(aes_reader))
        }
    }
}

/// Reader returned when a record decoder cannot be built: every access fails with the
/// original error, which binrw then reports as a parse error for this record.
struct FailedDecoder {
    kind: ErrorKind,
    message: String,
}

impl From<Error> for FailedDecoder {
    fn from(error: Error) -> Self {
        FailedDecoder {
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}

impl FailedDecoder {
    fn error(&self) -> Error {
        Error::new(self.kind, self.message.clone())
    }
}

impl Read for FailedDecoder {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
        Err(self.error())
    }
}

impl Seek for FailedDecoder {
    fn seek(&mut self, _pos: SeekFrom) -> Result<u64> {
        Err(self.error())
    }
}

/// Xor Encoding is an internal data encoding method used starting v4
/// It doesn't require any external keychain
pub struct XorDecoder<R> {
    reader: R,
    key: [u8; 8],
    start_position: u64,
    decode_position: usize,
}

impl<R: Read + Seek> XorDecoder<R> {
    pub fn new(mut reader: R, record_type: u8) -> Result<Self> {
        let mut first_byte = [0u8];
        reader.read_exact(&mut first_byte)?;
        let first_byte = first_byte[0];

        let start_position = reader.stream_position()?;

        let magic: u64 = 0x123456789ABCDEF0;
        let key = crc64(
            first_byte.overflowing_add(record_type).0 as u64,
            &magic.overflowing_mul(first_byte as u64).0.to_le_bytes(),
        )
        .to_le_bytes();

        Ok(XorDecoder {
            reader,
            key,
            start_position,
            decode_position: 0,
        })
    }
}

impl<R: Read> Read for XorDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let bytes_read = self.reader.read(buf)?;
        for (i, byte) in buf.iter_mut().enumerate().take(bytes_read) {
            *byte ^= self.key[(self.decode_position + i) % 8];
        }
        self.decode_position += bytes_read;
        Ok(bytes_read)
    }
}

impl<R: Seek> Seek for XorDecoder<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(position) => {
                let offset = position.checked_sub(self.start_position).ok_or_else(|| {
                    Error::new(ErrorKind::InvalidInput, "seek before start of record")
                })?;
                self.decode_position = offset as usize;
                self.reader.seek(pos)
            }
            SeekFrom::Current(_) => self.reader.seek(pos),
            _ => Err(Error::other("Unsupported seek")),
        }
    }
}

pub struct AesDecoder {
    buffer: Cursor<Vec<u8>>,
    pub next_iv: Vec<u8>,
}

impl AesDecoder {
    /// Reads and decrypts `size` bytes from `reader`.
    ///
    /// The plaintext is exposed as a stream of `stream_size` bytes, padded with zeros: some record
    /// structures are parsed over the full record size, which is larger than the encrypted content.
    pub fn new<R: Read>(
        mut reader: R,
        iv: &[u8],
        key: &[u8],
        size: u16,
        stream_size: u16,
    ) -> Result<AesDecoder> {
        let mut buffer = vec![0u8; size.into()];
        reader.read_exact(&mut buffer)?;

        // Next iv is the last ciphertext block
        let next_iv_start = buffer
            .len()
            .checked_sub(Aes256::block_size())
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "encrypted record too short"))?;
        let next_iv = buffer[next_iv_start..].to_vec();

        let decryptor = Aes256CbcDec::new_from_slices(key, iv)
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "invalid AES key or iv length"))?;
        // Some valid records have an invalid PKCS7 padding: they are kept, read as zeros, so the
        // iv chain (and therefore the following records) stays in sync
        let plaintext_len = decryptor
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .map(|plaintext| plaintext.len())
            .unwrap_or(0);

        buffer.truncate(plaintext_len);
        buffer.resize(stream_size.max(size).into(), 0);

        Ok(AesDecoder {
            buffer: Cursor::new(buffer),
            next_iv,
        })
    }
}

impl Read for AesDecoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.buffer.read(buf)
    }
}

impl Seek for AesDecoder {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.buffer.seek(pos)
    }
}
