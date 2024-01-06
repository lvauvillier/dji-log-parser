use binrw::binread;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct OFDM {
    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x7F)))]
    pub signal_percent: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80) == 1))]
    pub is_up: bool,
}
