use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameRC {
    /// Downlink signal strength
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub downlink_signal: Option<u8>,
    /// Uplink signal strength
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
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
