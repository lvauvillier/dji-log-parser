use crc64::crc64;
use std::io::{self, Read};

pub struct Decoder<R: Read> {
    inner: R,
    key: [u8; 8],
    index: usize,
}

impl<R: Read> Decoder<R> {
    pub fn new(mut reader: R, record_type: u8) -> Self {
        let mut first_byte = [0u8];
        reader.read_exact(&mut first_byte).unwrap();
        let first_byte = first_byte[0];

        let magic: u64 = 0x123456789ABCDEF0;
        let key = crc64(
            first_byte.overflowing_add(record_type).0 as u64,
            &magic.overflowing_mul(first_byte as u64).0.to_le_bytes(),
        )
        .to_le_bytes();

        Decoder {
            inner: reader,
            key,
            index: 0,
        }
    }
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        for i in 0..bytes_read {
            buf[i] ^= self.key[(self.index + i) % 8];
        }
        self.index += bytes_read;

        Ok(bytes_read)
    }
}
