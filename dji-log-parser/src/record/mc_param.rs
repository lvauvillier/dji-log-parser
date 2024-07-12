use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct MCParams {
    #[br(map = |x:u8| FailSafeProtectionType::from(x))]
    pub fail_safe_protection: FailSafeProtectionType,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01) == 1))]
    pub mvo_func_enabled: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x02) == 1))]
    pub avoid_obstacle_enabled: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x04) == 1))]
    pub user_avoid_enabled: bool,
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum FailSafeProtectionType {
    Hover,
    Landing,
    GoHome,
    #[serde(untagged)]
    Unknown(u8),
}

impl FailSafeProtectionType {
    fn from(value: u8) -> Self {
        match value {
            0 => FailSafeProtectionType::Hover,
            1 => FailSafeProtectionType::Landing,
            2 => FailSafeProtectionType::GoHome,
            _ => FailSafeProtectionType::Unknown(value),
        }
    }
}
