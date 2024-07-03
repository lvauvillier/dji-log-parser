use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
pub struct Firmware {
    #[br(map = |x: u8| SenderType::from(x))]
    pub sender_type: SenderType,
    pub sub_sender_type: u8,
    #[br(map = |x: [u8; 4]| format!("{}.{}.{}", x[0], x[1], x[2]))]
    pub version: String,
}

#[derive(Serialize, Debug)]
pub enum SenderType {
    None,
    Camera,
    MC,
    Gimbal,
    RC,
    Battery,
    Unknown(u8),
}

impl SenderType {
    pub fn from(value: u8) -> Self {
        match value {
            0 => SenderType::None,
            1 => SenderType::Camera,
            3 => SenderType::MC,
            4 => SenderType::Gimbal,
            6 => SenderType::RC,
            11 => SenderType::Battery,
            _ => SenderType::Unknown(value),
        }
    }
}
