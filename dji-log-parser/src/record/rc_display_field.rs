use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
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
