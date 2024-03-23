use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
pub struct Firmware {
    pub sender_type: u8,
    pub sub_sender_type: u8,
    #[br(map = |x: [u8; 4]| format!("{}.{}.{}", x[0], x[1], x[2]))]
    pub version: String,
}
