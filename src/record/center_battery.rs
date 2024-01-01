use binrw::binread;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct CenterBattery {
    pub relative_capacity: u8,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltage: f32,
    pub charge_remaining: u16,
    pub full_capacity: u16,
    pub life: u8,
    pub number_of_discharges: u16,
    pub error_type: u32,
    #[br(map = |x: i16| x as f32 / 1000.0)]
    pub current: f32,

    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltageCell1: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltageCell2: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltageCell3: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltageCell4: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltageCell5: f32,
    #[br(map = |x: u16| x as f32 / 1000.0)]
    pub voltageCell6: f32,

    pub serial_number: u16,
    pub productDate: u16,
    #[br(map = |x: u16| x as f32 / 10.0 - 273.15)]
    pub temperature: f32,
    pub connect_state: u8,
    pub sum_learn_count: u16,
    pub latest_learn_cycle: u16,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01)))]
    pub battery_on_charge: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0xFE)))]
    pub reverse: u8,
}
