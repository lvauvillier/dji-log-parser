use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
pub struct AppGPS {
    /// degrees
    pub longitude: f64,
    /// degrees
    pub latitude: f64,
}
