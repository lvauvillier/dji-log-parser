use binrw::binread;
use serde::Serialize;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
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

#[derive(Serialize, Debug)]
pub enum DeformMode {
    Pack,
    Protect,
    Normal,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for DeformMode {
    fn from(value: u8) -> Self {
        match value {
            0 => DeformMode::Pack,
            1 => DeformMode::Protect,
            2 => DeformMode::Normal,
            _ => DeformMode::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug)]
pub enum DeformStatus {
    FoldComplete,
    Folding,
    StretchComplete,
    Stretching,
    StopDeformation,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for DeformStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => DeformStatus::FoldComplete,
            2 => DeformStatus::Folding,
            3 => DeformStatus::StretchComplete,
            4 => DeformStatus::Stretching,
            5 => DeformStatus::StopDeformation,
            _ => DeformStatus::Unknown(value),
        }
    }
}
