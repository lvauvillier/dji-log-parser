use binrw::binread;
use serde::Serialize;
use std::f64::consts::PI;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;
use num_enum::FromPrimitive;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Home {
    /// degrees
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub longitude: f64,
    /// degrees
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub latitude: f64,
    /// meters
    #[br(map = |x: f32| x / 10.0)]
    pub altitude: f32,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01) == 1))]
    pub is_home_record: bool,
    #[br(calc(GoHomeMode::from(sub_byte_field(_bitpack1, 0x02) == 1)))]
    pub go_home_mode: GoHomeMode,
    #[br(calc(sub_byte_field(_bitpack1, 0x04)))]
    pub aircraft_head_direction: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x08) == 1))]
    pub is_dynamic_home_point_enabled: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x10) == 1))]
    pub is_near_distance_limit: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x20) == 1))]
    pub is_near_height_limit: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x40) == 1))]
    pub is_multiple_mode_open: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x80) == 1))]
    pub has_go_home: bool,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(CompassCalibrationState::from(sub_byte_field(_bitpack2, 0x03))))]
    pub compass_state: CompassCalibrationState,
    #[br(calc(sub_byte_field(_bitpack2, 0x04) == 1))]
    pub is_compass_adjust: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x08) == 1))]
    pub is_beginner_mode: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x10) == 1))]
    pub is_ioc_open: bool,
    #[br(calc(IOCMode::from(sub_byte_field(_bitpack2, 0xE0))))]
    pub ioc_mode: IOCMode,

    pub go_home_height: u16,
    pub ioc_course_lock_angle: i16,
    pub flight_record_sd_state: u8,
    pub record_sd_capacity_percent: u8,
    pub record_sd_left_time: u16,
    pub current_flight_record_index: u16,
    #[br(if(version >= 8), temp)]
    _unknown: [u8; 5],
    #[br(if(version >= 8))]
    pub max_allowed_height: f32,
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum IOCMode {
    CourseLock = 1,
    HomeLock = 2,
    HotspotSurround = 3,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum GoHomeMode {
    Normal,
    FixedHeight,
}

impl From<bool> for GoHomeMode {
    fn from(value: bool) -> Self {
        match value {
            false => GoHomeMode::Normal,
            true => GoHomeMode::FixedHeight,
        }
    }
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum CompassCalibrationState {
    NotCalibrating = 0,
    Horizontal = 1,
    Vertical = 2,
    Successful = 3,
    Failed = 4,
    #[num_enum(catch_all)]
    Unknown(u8),
}
