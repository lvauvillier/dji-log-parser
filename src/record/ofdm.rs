use binrw::binread;
use serde::Serialize;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
pub struct OFDM {
    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x7F)))]
    pub signal_percent: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80) == 1))]
    pub is_up: bool,
}
