use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;
use num_enum::FromPrimitive;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Deform {
    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01) == 1))]
    pub is_deform_protected: bool,
    #[br(calc(DeformStatus::from(sub_byte_field(_bitpack1, 0x0E))))]
    pub deform_status: DeformStatus,
    #[br(calc(DeformMode::from(sub_byte_field(_bitpack1, 0x30))))]
    pub deform_mode: DeformMode,
}

#[derive(FromPrimitive, Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum DeformMode {
    Pack = 0,
    Protect = 1,
    Normal = 2,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum DeformStatus {
    FoldComplete = 1,
    Folding = 2,
    StretchComplete = 3,
    Stretching = 4,
    StopDeformation = 5,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}
