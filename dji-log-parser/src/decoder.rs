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
/// This function returns a `Result` containing either a boxed reader implementing `Read`
/// or an error if one occurs during the reader's construction.
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
    match version {
        // Raw
        0..=6 => Box::new(reader),
        // Xor
        7..=12 => Box::new(XorDecoder::new(reader, record_type)),
        // Xor + AES
        _ => {
            let feature_point = FeaturePoint::from_record_type(record_type, version);
            match feature_point {
                FeaturePoint::PlaintextFeature => Box::new(XorDecoder::new(reader, record_type)),
                _ => {
                    let pair = keychain
                        .borrow()
                        .get(&feature_point)
                        .map(|value| (value.0.clone(), value.1.clone()));

                    match pair {
                        Some(value) => {
                            let aes_reader = AesDecoder::new(
                                XorDecoder::new(reader, record_type),
                                &value.0,
                                &value.1,
                                size - 2, // firstChar and lastChar are not part of the content
                            );

                            // Update keychain with next iv
                            keychain.borrow_mut().insert(
                                feature_point,
                                (aes_reader.next_iv.clone(), value.1.clone()),
                            );

                            Box::new(aes_reader)
                        }
                        None => Box::new(XorDecoder::new(reader, record_type)),
                    }
                }
            }
        }
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
    pub fn new(mut reader: R, record_type: u8) -> Self {
        let mut first_byte = [0u8];
        reader.read_exact(&mut first_byte).unwrap();
        let first_byte = first_byte[0];

        let start_position = reader.stream_position().unwrap();

        let magic: u64 = 0x123456789ABCDEF0;
        let key = crc64(
            first_byte.overflowing_add(record_type).0 as u64,
            &magic.overflowing_mul(first_byte as u64).0.to_le_bytes(),
        )
        .to_le_bytes();

        XorDecoder {
            reader,
            key,
            start_position,
            decode_position: 0,
        }
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
                self.decode_position = (position - self.start_position) as usize;
                self.reader.seek(pos)
            }
            SeekFrom::Current(_) => self.reader.seek(pos),
            _ => Err(Error::new(ErrorKind::Other, "Unsupported seek")),
        }
    }
}

pub struct AesDecoder {
    buffer: Cursor<Vec<u8>>,
    pub next_iv: Vec<u8>,
}

impl AesDecoder {
    pub fn new<R: Read>(mut reader: R, iv: &[u8], key: &[u8], size: u16) -> AesDecoder {
        let mut buffer = vec![0u8; size.into()];
        reader.read_exact(&mut buffer).unwrap();

        // Get next from last block
        let next_iv = buffer[buffer.len() - Aes256::block_size()..].to_vec();

        let dec: cbc::Decryptor<Aes256> = Aes256CbcDec::new_from_slices(key, iv).unwrap();
        let plaintext = dec
            .decrypt_padded_mut::<Pkcs7>(&mut buffer)
            .unwrap_or_default()
            .to_vec();

        AesDecoder {
            buffer: Cursor::new(plaintext.to_vec()),
            next_iv,
        }
    }
}

impl Read for AesDecoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let _ = self.buffer.read(buf);
        Ok(buf.len()) // always return buffer length to avoid padding issues
    }
}

impl Seek for AesDecoder {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.buffer.seek(pos)
    }
}
