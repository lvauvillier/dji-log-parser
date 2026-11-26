use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameApp {
    // App tip
    pub tip: String,
    // App warning
    pub warn: String,
    /// Right stick horizontal position (aileron) of the app, from 364 to 1684, 1024 is the center
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub aileron: Option<u16>,
    /// Right stick vertical position (elevator) of the app
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub elevator: Option<u16>,
    /// Left stick vertical position (throttle) of the app
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub throttle: Option<u16>,
    /// Left stick horizontal position (rudder) of the app
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub rudder: Option<u16>,
}
