use serde::Serialize;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RC {
    /// Downlink signal strength
    pub downlink_signal: Option<u8>,
    /// Uplink signal strength
    pub uplink_signal: Option<u8>,
    /// Right stick horizontal position (aileron)
    pub aileron: u16,
    /// Right stick vertical position (elevator)
    pub elevator: u16,
    /// Left stick vertical position (throttle)
    pub throttle: u16,
    /// Left stick horizontal position (rudder)
    pub rudder: u16,
}
