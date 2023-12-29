use binrw::binread;

// Constants
const OLD_PREFIX_SIZE: u64 = 12;
const PREFIX_SIZE: u64 = 100;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Prefix {
    detail_offset: u64,
    detail_length: u16,
    pub version: u8,
    #[br(temp)]
    unknown: u8,
    encrypt_magic_version: u64,
    #[br(temp)]
    reserved: [u8; 80],
}

impl Prefix {
    pub fn info_offset(&self) -> u64 {
        if self.version < 12 {
            self.detail_offset
        } else {
            PREFIX_SIZE
        }
    }

    pub fn records_offset(&self) -> u64 {
        if self.version < 6 {
            OLD_PREFIX_SIZE
        } else if self.version < 12 {
            PREFIX_SIZE
        } else if self.version == 12 {
            PREFIX_SIZE + 436 // We manually add info size
        } else {
            self.detail_offset
        }
    }

    pub fn image_offset(&self) -> u64 {
        todo!()
    }
}
