use binrw::binread;

#[binread]
#[derive(Debug, Clone)]
#[br(little)]
pub struct Prefix {
    pub detail_offset: u64,
    pub detail_length: u16,
    pub format_version: u8,
}
