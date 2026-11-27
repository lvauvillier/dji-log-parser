use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;

/// Signal of the radio link, for the uplink or the downlink.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct OFDM {
    #[br(temp)]
    _bitpack1: u8,
    /// Raw value: a percentage on Wi-Fi and Lightbridge links, levels from 0 to 90 on OcuSync links
    #[br(calc(sub_byte_field(_bitpack1, 0x7F)))]
    pub signal: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80) == 1))]
    pub is_up: bool,
}
