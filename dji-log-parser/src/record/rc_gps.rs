use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
pub struct RCGPS {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub latitude: i32,
    pub longitude: i32,
    pub speed_x: i32,
    pub speed_y: i32,
    pub gps_num: u8,
    pub accuracy: f32,
    pub valid_data: u16,
}
