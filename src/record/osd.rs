use binrw::binread;
use std::f64::consts::PI;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct OSD {
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub longitude: f64,
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub lattitude: f64,

    pub barometer_height: i16,
    pub speed_x: i16,
    pub speed_y: i16,
    pub speed_z: i16,
    pub pitch: i16,
    pub roll: i16,
    pub yaw: i16,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x7F)))]
    pub control_mode: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80)))]
    pub rc_outcontrol: u8,

    pub app_command: u8,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x01)))]
    pub can_ioc_work: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x06)))]
    pub ground_or_sky: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x08)))]
    pub is_motor_up: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x10)))]
    pub is_swave_work: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0xE0)))]
    pub go_home_status: u8,

    #[br(temp)]
    _bitpack3: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x01)))]
    pub is_vision_used: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x06)))]
    pub voltage_warning: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x10)))]
    pub is_imu_preheated: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x60)))]
    pub mode_channel: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x80)))]
    pub is_gps_valid: u8,

    #[br(temp)]
    _bitpack4: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x01)))]
    pub is_compass_error: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x02)))]
    pub wave_error: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x3C)))]
    pub gps_level: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0xC0)))]
    pub battery_type: u8,

    #[br(temp)]
    _bitpack5: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x01)))]
    pub is_out_of_limit: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x02)))]
    pub is_go_home_height_modified: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x04)))]
    pub is_propeller_catapult: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x08)))]
    pub is_motor_blocked: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x10)))]
    pub is_not_enough_force: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x20)))]
    pub is_barometer_dead_in_air: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x40)))]
    pub is_vibrating: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x80)))]
    pub is_acceletor_over_range: u8,

    pub gps_num: u8,
    pub flight_action: u8,
    pub motor_start_failed_cause: u8,

    #[br(temp)]
    _bitpack6: u8,
    #[br(calc(sub_byte_field(_bitpack6, 0x0F)))]
    pub non_gpscause: u8,
    #[br(calc(sub_byte_field(_bitpack6, 0x10)))]
    pub waypoint_limit_mode: u8,

    pub battery: u8,
    pub sWaveHeight: u8,
    pub flyTime: u16,
    pub motorRevolution: u8,
    #[br(temp)]
    _unknown: u16,
    pub version: u8,
    pub droneType: u8,
}
