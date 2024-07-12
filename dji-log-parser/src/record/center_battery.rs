use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct CenterBattery {
    pub relative_capacity: u8,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage: f32,
    pub current_capacity: u16,
    pub full_capacity: u16,
    pub life: u8,
    pub number_of_discharges: u16,
    pub error_type: u32,
    #[br(map = |x: i16| x as f32 / 1000.0)]
    pub current: f32,

    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage_cell1: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage_cell2: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage_cell3: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage_cell4: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage_cell5: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage_cell6: f32,

    pub serial_number: u16,
    pub product_date: u16,
    #[br(if(version >=8), map = |x: u16| x as f32 / 10.0 - 273.15)]
    pub temperature: f32,
    #[br(if(version >=8))]
    pub connect_state: u8,
    #[br(if(version >=8))]
    pub sum_learn_count: u16,
    #[br(if(version >=8))]
    pub latest_learn_cycle: u16,

    #[br(if(version >=8), temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01) == 1))]
    pub battery_on_charge: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0xFE)))]
    pub reverse: u8,
}
