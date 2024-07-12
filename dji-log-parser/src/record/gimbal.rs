use crate::utils::sub_byte_field;
use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(import { version: u8 }, little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Gimbal {
    /// degrees
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub pitch: f32,
    /// degrees
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub roll: f32,
    /// degrees
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub yaw: f32,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(GimbalMode::from(sub_byte_field(_bitpack1, 0xC0))))]
    pub mode: GimbalMode,
    #[br(calc(sub_byte_field(_bitpack1, 0x20)))]
    pub reset: u8,

    #[br(map = |x: i8| (x as f32 / 10.0))]
    pub roll_adjust: f32,
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub yaw_angle: f32,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x01) == 1))]
    pub is_pitch_at_limit: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x02) == 1))]
    pub is_roll_at_limit: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x04) == 1))]
    pub is_yaw_at_limit: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x08) == 1))]
    pub is_auto_calibration: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x10) == 1))]
    pub auto_calibration_result: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x20) == 1))]
    pub install_direction: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x40) == 1))]
    pub is_stuck: bool,
    #[br(if(version >=2), temp)]
    _bitpack3: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x0F)))]
    pub version: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x20) == 1))]
    pub is_double_click: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x40) == 1))]
    pub is_triple_click: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x80) == 1))]
    pub is_single_click: bool,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum GimbalMode {
    /// The gimbal can move independently of the aircraft's yaw. In this mode, even if
    /// the aircraft yaw changes, the camera will continue pointing in the same world
    /// direction. This mode is only available for the Ronin-MX when the M600 or M600
    /// Pro landing gear is retracted.
    Free,
    /// The gimbal's work mode is FPV mode. In this mode, the gimbal yaw will follow the
    /// aircraft's heading, and the gimbal roll will follow the RC's roll channel value.
    /// The pitch will be available to move.
    /// This mode is only available for the Ronin-MX when the M600 landing gear is retracted.
    FPV,
    /// The gimbal's work mode is such that it will follow the yaw. In this mode, the
    /// gimbal yaw will be fixed, while pitch and roll will be available to move.
    YawFollow,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for GimbalMode {
    fn from(value: u8) -> Self {
        match value {
            0 => GimbalMode::Free,
            1 => GimbalMode::FPV,
            2 => GimbalMode::YawFollow,
            _ => GimbalMode::Unknown(value),
        }
    }
}
