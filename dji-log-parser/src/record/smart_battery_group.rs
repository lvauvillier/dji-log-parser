use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum SmartBatteryGroup {
    #[br(magic = 1u8)]
    SmartBatteryStatic(SmartBatteryStatic),
    #[br(magic = 2u8)]
    SmartBatteryDynamic(SmartBatteryDynamic),
    #[br(magic = 3u8)]
    SmartBatterySingleVoltage(SmartBatterySingleVoltage),
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatteryStatic {
    pub index: u8,
    pub designed_capacity: u32,
    pub loop_times: u16,
    pub full_voltage: u32,
    #[br(temp)]
    _unknown: u16,
    serial_number: u16,
    #[br(temp)]
    _unknown2: [u8; 10],
    #[br(temp)]
    _unknown3: [u8; 5],
    pub version_number: [u8; 8],
    pub battery_life: u8,
    pub battery_type: u8,
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatteryDynamic {
    pub index: u8,
    /// volts
    #[br(map = |x: i32| x as f32 / 1000.0)]
    pub current_voltage: f32,
    // ampere
    #[br(map = |x: i32| x.abs() as f32 / 1000.0)]
    pub current_current: f32,
    /// mAh
    pub full_capacity: u32,
    /// mAh
    pub remained_capacity: u32,
    /// degrees
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub temperature: f32,
    pub cell_count: u8,
    pub capacity_percent: u8,
    pub battery_state: u64,
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatterySingleVoltage {
    pub index: u8,
    pub cell_count: u8,
    #[br(count = cell_count, map = |xs: Vec<u16>| xs.into_iter().map(|x| x as f32 / 1000.0).collect())]
    pub cell_voltages: Vec<f32>,
}
