use std::f64::consts::PI;

use crate::utils::sub_byte_field;
use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct Home {
    /// degrees
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub longitude: f64,
    /// degrees
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub lattitude: f64,
    /// meters
    #[br(map = |x: f32| (x / 10.0))]
    pub altitude: f32,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01)))]
    pub is_home_record: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x02)))]
    pub go_home_mode: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x04)))]
    pub aircraft_head_direction: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x08)))]
    pub is_dynamic_home_point_enabled: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x10)))]
    pub is_near_distance_limit: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x20)))]
    pub is_near_height_limit: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x40)))]
    pub is_multiple_mode_open: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80)))]
    pub has_go_home: u8,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x03)))]
    pub compass_state: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x04)))]
    pub is_compass_adjust: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x08)))]
    pub is_beginner_mode: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x10)))]
    pub is_ioc_open: u8,
    #[br(calc(IOCMode::from(sub_byte_field(_bitpack2, 0xE0))))]
    pub ioc_mode: IOCMode,

    pub go_home_height: u16,
    pub ioc_course_lock_angle: i16,
    pub flight_record_sd_state: u8,
    pub record_sd_capacity_percent: u8,
    pub record_sd_left_time: u16,
    pub current_flight_record_index: u16,
    #[br(temp)]
    pub unknown: [u8; 5],
    pub max_allowed_height: f32,
}

#[derive(Debug)]
pub enum IOCMode {
    CourseLock,
    HomeLock,
    HotspotSurround,
    Unknown(u8),
}

impl From<u8> for IOCMode {
    fn from(value: u8) -> Self {
        match value {
            1 => IOCMode::CourseLock,
            2 => IOCMode::HomeLock,
            3 => IOCMode::HotspotSurround,
            _ => IOCMode::Unknown(value),
        }
    }
}
