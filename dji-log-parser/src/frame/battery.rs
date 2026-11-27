use serde::Serialize;

use crate::record::smart_battery::BatteryGoHomeStatus;

use crate::record::smart_battery_group::SmartBatteryDynamic;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameBattery {
    /// Battery charge level in percentage
    pub charge_level: u8,
    /// Battery voltage
    pub voltage: f32,
    /// Battery current
    pub current: f32,
    /// Current battery capacity
    pub current_capacity: u32,
    /// Full battery capacity
    pub full_capacity: u32,
    /// Number of battery cells
    pub cell_num: u8,
    /// Indicates if cell voltage is derived from global voltage
    pub is_cell_voltage_estimated: bool,
    /// Cell voltages
    pub cell_voltages: Vec<f32>,
    /// Deviation in cell voltages
    pub cell_voltage_deviation: f32,
    /// Maximum deviation in cell voltages
    pub max_cell_voltage_deviation: f32,
    /// Battery temperature
    pub temperature: f32,
    /// Minimum battery temperature
    pub min_temperature: f32,
    /// Maximum battery temperature
    pub max_temperature: f32,
    /// Flight time left, in seconds
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub useful_time: Option<u16>,
    /// Time needed to return home, in seconds
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_time: Option<u16>,
    /// Time needed to land, in seconds
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub land_time: Option<u16>,
    /// Battery needed to return home, in percentage
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_battery: Option<u16>,
    /// Battery needed to land, in percentage
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub land_battery: Option<u16>,
    /// Low battery warning threshold, in percentage
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub low_warning: Option<u8>,
    /// Critical battery warning threshold, in percentage
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub serious_low_warning: Option<u8>,
    /// Status bits of the battery: bit 0 is the low warning and bit 1 the critical warning, the
    /// others are not known
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub status: Option<u32>,
    /// Return home status of the battery
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_status: Option<BatteryGoHomeStatus>,
    /// Countdown before the return home, in seconds, none when there is no countdown
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_countdown: Option<u8>,
    /// Indicates if the aircraft returns home at the low battery warning
    pub low_warning_go_home: bool,
    /// Indicates if the aircraft lands at the critical battery warning
    pub serious_low_warning_landing: bool,
}

/// One battery of a drone with several batteries, the others values of the frame being the total
/// of all batteries.
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameBatteryPack {
    /// Battery charge level in percentage
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub charge_level: Option<u8>,
    /// Battery voltage
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub voltage: Option<f32>,
    /// Battery current
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub current: Option<f32>,
    /// Current battery capacity
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub current_capacity: Option<u32>,
    /// Full battery capacity
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub full_capacity: Option<u32>,
    /// Battery temperature
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub temperature: Option<f32>,
}

impl FrameBatteryPack {
    pub(crate) fn update(&mut self, battery: &SmartBatteryDynamic) {
        self.charge_level = Some(battery.capacity_percent);
        self.voltage = Some(battery.current_voltage);
        self.current = Some(battery.current_current);
        self.current_capacity = Some(battery.remained_capacity);
        self.full_capacity = Some(battery.full_capacity);
        self.temperature = Some(battery.temperature);
    }
}
