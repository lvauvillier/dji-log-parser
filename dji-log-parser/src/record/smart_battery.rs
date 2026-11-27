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
pub struct SmartBattery {
    pub useful_time: u16,
    pub go_home_time: u16,
    pub land_time: u16,
    pub go_home_battery: u16,
    pub land_battery: u16,
    /// Meaning unknown: values up to hundreds of thousands, not a radius
    pub safe_fly_radius: f32,
    /// Meaning unknown: values up to tens of thousands
    pub volume_consume: f32,
    /// Bits of the battery warnings
    pub status: u32,
    /// Indicates if the low battery warning is on: the charge reached `low_warning`
    #[br(calc(status & 0x01 != 0))]
    pub is_low_warning: bool,
    /// Indicates if the critical battery warning is on: the charge reached `serious_low_warning`
    #[br(calc(status & 0x02 != 0))]
    pub is_serious_low_warning: bool,
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

#[derive(FromPrimitive, Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum BatteryGoHomeStatus {
    NonGoHome = 0,
    GoHome = 1,
    GoHomeAlready = 2,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::BinRead;

    #[test]
    fn warning_bits_of_the_status() {
        let mut data = [0u8; 30];
        // Status 0x1003: bit 12 is set too, its meaning is unknown
        data[18..22].copy_from_slice(&0x1003u32.to_le_bytes());

        let battery = SmartBattery::read_le(&mut Cursor::new(data)).unwrap();

        assert!(battery.is_low_warning);
        assert!(battery.is_serious_low_warning);
    }
}
