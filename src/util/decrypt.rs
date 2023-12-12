use crc64::crc64;

/// Decrypts data based on the record type and input data.
///
/// This function applies an internal decryption algorithm to the given data slice based on the record type.
/// It returns the decrypted data as a `Vec<u8>`.
pub fn decrypt(record_type: u8, data: &[u8]) -> Vec<u8> {
    let mut decrypted_data = Vec::with_capacity(data.len() - 1);

    let magic: u64 = 0x123456789ABCDEF0;
    let key = crc64(
        data[0].overflowing_add(record_type).0 as u64,
        &magic.overflowing_mul(data[0] as u64).0.to_le_bytes(),
    )
    .to_le_bytes();

    for (index, &element) in data[1..].iter().enumerate() {
        decrypted_data.push(element ^ key[index % 8]);
    }

    decrypted_data
}
