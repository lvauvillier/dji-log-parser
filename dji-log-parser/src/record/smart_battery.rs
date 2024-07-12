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
pub struct SmartBattery {
    pub useful_time: u16,
    pub go_home_time: u16,
    pub land_time: u16,
    pub go_home_battery: u16,
    pub land_battery: u16,
    pub safe_fly_radius: f32,
    pub volume_consume: f32,
    pub status: u32,
    #[br(map = |x: u8| BatteryGoHomeStatus::from(x))]
    pub go_home_status: BatteryGoHomeStatus,
    pub go_home_countdown: u8,
    /// volts
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage: f32,
    pub percent: u8,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x7F)))]
    pub low_warning: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80)))]
    pub low_warning_go_home: u8,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x7F)))]
    pub serious_low_warning: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x80)))]
    pub serious_low_warning_landing: u8,

    pub reserve: u8,
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum BatteryGoHomeStatus {
    NonGoHome,
    GoHome,
    GoHomeAlready,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for BatteryGoHomeStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => BatteryGoHomeStatus::NonGoHome,
            1 => BatteryGoHomeStatus::GoHome,
            2 => BatteryGoHomeStatus::GoHomeAlready,
            _ => BatteryGoHomeStatus::Unknown(value),
        }
    }
}
