use binrw::binread;
use chrono::{DateTime, NaiveDateTime};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// Position of the drone and of its home point, written by enterprise drones.
///
/// Older records end after the distance to the home point.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { length: u16 })]
#[br(assert(length >= MIN_LENGTH, "too short ground station data"))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct GroundStationPushData {
    /// degrees
    #[br(map = f64::to_degrees)]
    pub longitude: f64,
    /// degrees
    #[br(map = f64::to_degrees)]
    pub latitude: f64,
    /// meters, above the home point
    pub height: f32,
    /// degrees
    #[br(map = |x: i16| f32::from(x) / 10.0)]
    pub yaw: f32,
    #[br(temp)]
    _unknown: u16,
    #[br(temp)]
    seconds: u32,
    #[br(temp)]
    nanoseconds: u32,
    /// Time of the clock of the drone, in a time zone that isn't known: often the local time
    #[br(calc = local_time(seconds, nanoseconds))]
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    pub local_time: NaiveDateTime,
    #[br(temp)]
    _unknown2: u8,
    pub gps_num: u8,
    /// degrees
    #[br(map = f64::to_degrees)]
    pub home_longitude: f64,
    /// degrees
    #[br(map = f64::to_degrees)]
    pub home_latitude: f64,
    /// meters, above the WGS84 ellipsoid
    pub home_ellipsoid_height: f32,
    /// meters, horizontal distance to the home point
    pub home_distance: f32,
    #[br(temp)]
    _unknown3: [u8; 8],
    /// meters, height of the geoid above the WGS84 ellipsoid: the altitude above the sea level is
    /// the ellipsoid height minus the geoid height
    #[br(if(length >= GEOID_END))]
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub geoid_height: Option<f32>,
}

/// Length up to the distance to the home point
const MIN_LENGTH: u16 = 58;
/// Length up to the geoid height
const GEOID_END: u16 = 70;

fn local_time(seconds: u32, nanoseconds: u32) -> NaiveDateTime {
    DateTime::from_timestamp(seconds.into(), nanoseconds)
        .unwrap_or_default()
        .naive_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::args;
    use binrw::io::Cursor;
    use binrw::BinRead;

    fn read(hex: &str) -> binrw::BinResult<GroundStationPushData> {
        let mut data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let length = data.len() as u16;
        // Records are read followed by zeros
        data.resize(data.len() + 64, 0);
        GroundStationPushData::read_args(&mut Cursor::new(data), args! { length })
    }

    #[test]
    fn position_and_home_point() {
        let data = read(concat!(
            "daae16c08deecf3f739d403890e4ec3f662649429dfd01014d647e676d38480316272dc20d708deecf3f",
            "3ba1a04790e4ec3fd148df421d2c703e02ee214354431e4317d9284202000000000000000000",
        ))
        .unwrap();

        assert_eq!(data.longitude, 14.293439548254044);
        assert_eq!(data.latitude, 51.732405192162524);
        assert_eq!(data.height, 50.2875);
        assert_eq!(data.yaw, -61.1);
        assert_eq!(data.local_time.to_string(), "2025-01-08 11:41:01.055064685");
        assert_eq!(data.gps_num, 39);
        assert_eq!(data.home_longitude, 14.293437412889812);
        assert_eq!(data.home_latitude, 51.732406833014906);
        assert_eq!(data.home_ellipsoid_height, 111.64222);
        assert_eq!(data.home_distance, 0.23454328);
        assert_eq!(data.geoid_height, Some(42.212));
    }

    #[test]
    fn older_records_end_after_the_distance_to_the_home_point() {
        let data = read(concat!(
            "8dfe1a15456cfd3f8dc8f9f70071d73f9c484143fd00000066b2e6675ab56030090c71b3ebf3446cfd3f",
            "8dfc728a0171d73ffb8bba4226d4913f4d478f43",
        ))
        .unwrap();

        assert_eq!(data.home_distance, 1.1392868);
        assert_eq!(data.geoid_height, None);
    }

    #[test]
    fn shorter_records_are_rejected() {
        assert!(read("8dfe1a15456cfd3f8dc8f9f70071d73f").is_err());
    }
}
