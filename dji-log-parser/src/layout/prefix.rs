use binrw::binread;

// Constants
const OLD_PREFIX_SIZE: u64 = 12;
const PREFIX_SIZE: u64 = 100;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Prefix {
    detail_offset: u64,
    _detail_length: u16,
    pub version: u8,
    #[br(temp)]
    _unknown: u8,
    #[br(temp)]
    _encrypt_magic_version: u64,
    #[br(temp)]
    _reserved: [u8; 80],
}

impl Prefix {
    pub(crate) fn recover_detail_offset(&mut self, detail_offset: u64) {
        self.detail_offset = detail_offset;
    }

    pub(crate) fn detail_offset(&self) -> u64 {
        if self.version < 12 {
            self.detail_offset
        } else {
            PREFIX_SIZE
        }
    }

    pub(crate) fn records_offset(&self) -> u64 {
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

    pub(crate) fn records_end_offset(&self, file_size: impl Into<u64>) -> u64 {
        if self.version < 12 {
            self.detail_offset
        } else {
            file_size.into()
        }
    }
}
