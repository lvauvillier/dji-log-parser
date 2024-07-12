use serde::Serialize;
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
}
