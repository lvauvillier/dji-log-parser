use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::layout::details::ProductType;
use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { version: u8, product_type: ProductType = ProductType::None })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct RC {
    /// right stick - horizontal
    pub aileron: u16,
    /// right stick - vertical
    pub elevator: u16,
    /// left stick - vertical
    pub throttle: u16,
    /// left stick - horizontal
    pub rudder: u16,
    pub gimbal: u16,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01) == 1))]
    pub wheel_btn_down: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x3E)))]
    pub wheel_offset: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x40)))]
    pub wheel_polarity: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80)))]
    pub wheel_change: u8,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x07)))]
    pub transform_btn_reserve: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x08) == 1))]
    pub return_btn: bool,
    #[br(calc(FlightModeSwitch::from(sub_byte_field(_bitpack2, 0x30), product_type)))]
    pub flight_mode_switch: FlightModeSwitch,
    #[br(calc(sub_byte_field(_bitpack2, 0xC0)))]
    pub transform_switch: u8,

    #[br(temp)]
    _bitpack3: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x02) == 1))]
    pub custom_function_btn4_down: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x04) == 1))]
    pub custom_function_btn3_down: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x08) == 1))]
    pub custom_function_btn2_down: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x10) == 1))]
    pub custom_function_btn1_down: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x20) == 1))]
    pub playback_btn_down: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x40) == 1))]
    pub shutter_btn_down: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x80) == 1))]
    pub record_btn_down: bool,

    #[br(if(version >= 6))]
    pub bandwidth: u8,
    #[br(if(version >= 7))]
    pub gimbal_control_enable: u8,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum FlightModeSwitch {
    /// Position One. For all products except Mavic Pro, this is the left most position
    /// of the flight mode switch on a remote controller from the perspective of the
    /// pilot. For example, on a Phantom 4 remote controller,  Position One is labeled
    /// "A". For Mavic Pro, Spark and Mavic Air, this is  the position that is furthest
    /// away from the pilot and labeled "Sport".
    One,
    /// Position Two. For all products except Mavic Pro, this is the middle position of
    /// the flight mode switch on a remote controller from the perspective of the pilot.
    /// For example, on a Phantom 4 remote controller, Position Two is labeled "S". For
    /// Mavic Pro, Spark and Mavic Air, this is the position that is closest  to the
    /// pilot [the P position].
    Two,
    /// Position Three. For all products except Mavic Pro, this is the right most
    /// position of the flight mode switch on a remote controller from the perspective
    /// of the pilot. For example, on a Phantom 4 remote controller, Position Two is
    /// labeled "P". Mavic Pro, Spark or Mavic Air does not have  a third position for
    /// the flight mode switch.
    Three,
    #[serde(untagged)]
    Unknown(u8),
}

impl FlightModeSwitch {
    pub fn from(value: u8, product_type: ProductType) -> Self {
        let mapped_value = match product_type {
            // Remap values for Mavic Pro
            ProductType::MavicPro => match value {
                0 => 2,
                1 => 3,
                2 => 1,
                _ => value,
            },
            _ => value,
        };

        match mapped_value {
            0 => FlightModeSwitch::One,
            1 => FlightModeSwitch::Two,
            2 => FlightModeSwitch::Three,
            _ => FlightModeSwitch::Unknown(mapped_value),
        }
    }
}
