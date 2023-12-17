use binrw::binread;

// Constants
const OLD_PREFIX_SIZE: usize = 12;
const PREFIX_SIZE: usize = 100;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Prefix {
    detail_offset: u64,
    detail_length: u16,
    pub version: u8,
}

impl Prefix {
    pub fn info_offset(&self) -> usize {
        if self.version < 12 {
            self.detail_offset as usize
        } else {
            PREFIX_SIZE
        }
    }

    pub fn records_offset(&self) -> usize {
        if self.version < 6 {
            OLD_PREFIX_SIZE
        } else if self.version < 12 {
            PREFIX_SIZE
        } else if self.version == 12 {
            PREFIX_SIZE + 436 // We manually add info size
        } else {
            self.detail_offset as usize
        }
    }

    pub fn image_offset(&self) -> usize {
        todo!()
    }
}
