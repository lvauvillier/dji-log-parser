use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct AppGPS {
    /// degrees
    pub longitude: f64,
    /// degrees
    pub latitude: f64,
}
