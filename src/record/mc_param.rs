use binrw::binread;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Debug)]
#[br(little)]
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

#[derive(Debug)]
pub enum FailSafeProtectionType {
    Hover,
    Landing,
    GoHome,
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
