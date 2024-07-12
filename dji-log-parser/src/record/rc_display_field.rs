use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct RCDisplayField {
    #[br(temp)]
    _unknown: [u8; 7],
    /// right stick - horizontal
    pub aileron: u16,
    /// right stick - vertical
    pub elevator: u16,
    /// left stick - vertical
    pub throttle: u16,
    /// left stick - horizontal
    pub rudder: u16,
    pub gimbal: u16,
}
