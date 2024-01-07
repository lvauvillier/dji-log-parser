use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[br(little)]
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
pub struct SmartBatteryStatic {
    pub index: u8,
    pub designed_capacity: u32,
    pub loop_times: u16,
    pub full_voltage: u32,
    #[br(temp)]
    unknown: u16,
    serial_number: u16,
    #[br(temp)]
    unknown2: [u8; 10],
    #[br(temp)]
    unknown3: [u8; 5],
    pub version_number: [u8; 8],
    pub battery_life: u8,
    pub battery_type: u8,
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
pub struct SmartBatteryDynamic {
    pub index: u8,
    /// volts
    #[br(map = |x: i32| x as f64 / 1000.0)]
    pub current_voltage: f64,
    pub current_current: i32,
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
pub struct SmartBatterySingleVoltage {
    pub index: u8,
    pub cell_count: u8,
}
