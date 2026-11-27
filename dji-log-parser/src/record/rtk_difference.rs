use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// RTK positioning of the drone, about once per second.
///
/// Recent aircraft, such as the Matrice 400, add the QZSS satellites at the end.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct RTKDifference {
    #[br(temp)]
    _unknown: u8,
    #[br(map = |x: u8| RtkSolution::from(x))]
    pub solution: RtkSolution,
    #[br(temp)]
    satellites: [u8; 9],
    /// degrees
    pub latitude: f64,
    /// degrees
    pub longitude: f64,
    /// meters, above the WGS84 ellipsoid
    pub ellipsoid_height: f32,
    #[br(temp)]
    base_station_position: (f64, f64),
    /// degrees, none without a base station
    #[br(calc = base_station(base_station_position).map(|(latitude, _)| latitude))]
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub base_station_latitude: Option<f64>,
    /// degrees, none without a base station
    #[br(calc = base_station(base_station_position).map(|(_, longitude)| longitude))]
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub base_station_longitude: Option<f64>,
    /// meters, none without a base station
    #[br(map = |x: f32| (x != 0.0).then_some(x))]
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub base_station_altitude: Option<f32>,
    #[br(temp, pad_before = 14)]
    /// Standard deviations of the position: latitude, longitude and altitude
    deviations: [f32; 3],
    /// meters, standard deviation of the latitude
    #[br(calc = deviations[0])]
    pub std_latitude: f32,
    /// meters, standard deviation of the longitude
    #[br(calc = deviations[1])]
    pub std_longitude: f32,
    /// meters, standard deviation of the altitude
    #[br(calc = deviations[2])]
    pub std_altitude: f32,
    #[br(temp)]
    galileo: [u8; 3],
    /// Zeros in the records without them, read as unknown counts
    #[br(temp)]
    qzss: [u8; 3],

    /// Satellites of the main antenna of the drone
    #[br(calc = RtkSatellites::new(&satellites[0..3], galileo[0], qzss[0]))]
    pub antenna1_satellites: RtkSatellites,
    /// Satellites of the second antenna of the drone
    #[br(calc = RtkSatellites::new(&satellites[3..6], galileo[1], qzss[1]))]
    pub antenna2_satellites: RtkSatellites,
    /// Satellites of the base station
    #[br(calc = RtkSatellites::new(&satellites[6..9], galileo[2], qzss[2]))]
    pub base_station_satellites: RtkSatellites,
}

/// Position of the base station, none when it is 0.
fn base_station(position: (f64, f64)) -> Option<(f64, f64)> {
    (position != (0.0, 0.0)).then_some(position)
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum RtkSolution {
    None,
    /// Position of the receiver alone
    Single,
    Float,
    Fixed,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for RtkSolution {
    fn from(value: u8) -> Self {
        match value {
            0..=15 => RtkSolution::None,
            16 => RtkSolution::Single,
            17..=49 => RtkSolution::Float,
            50 => RtkSolution::Fixed,
            _ => RtkSolution::Unknown(value),
        }
    }
}

/// Number of satellites used, by constellation, none when unknown
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct RtkSatellites {
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub gps: Option<u8>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub beidou: Option<u8>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub glonass: Option<u8>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub galileo: Option<u8>,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub qzss: Option<u8>,
}

impl RtkSatellites {
    /// Reads the counts of GPS, BeiDou and GLONASS satellites, then Galileo and QZSS
    fn new(counts: &[u8], galileo: u8, qzss: u8) -> Self {
        RtkSatellites {
            gps: satellite_count(counts[0]),
            beidou: satellite_count(counts[1]),
            glonass: satellite_count(counts[2]),
            galileo: satellite_count(galileo),
            qzss: satellite_count(qzss),
        }
    }
}

/// A satellite count is stored shifted by one bit, the low bit tells that it is known
fn satellite_count(value: u8) -> Option<u8> {
    (value & 0x01 == 1).then_some(value >> 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::BinRead;

    fn read(hex: &str) -> binrw::BinResult<RTKDifference> {
        let mut data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        // Records are read followed by zeros
        data.resize(data.len() + 64, 0);
        RTKDifference::read_le(&mut Cursor::new(data))
    }

    #[test]
    fn fixed_solution_with_a_base_station() {
        let rtk = read(concat!(
            "003215110f15110f15090b534edae4920a4440fd4684f3e00c56c0fd8a3f432ba90072970a4440db",
            "c5c601df0c56c0f1b33d43c5dccf423201cccccc3f00000000ff325d3c2796263ca8a4ce3c0f0d07",
        ))
        .unwrap();

        assert_eq!(rtk.solution, RtkSolution::Fixed);
        assert_eq!(rtk.latitude, 40.0826078477809);
        assert_eq!(rtk.longitude, -88.20122993390756);
        assert_eq!(rtk.ellipsoid_height, 191.54292);
        assert_eq!(rtk.base_station_latitude, Some(40.0827467444636));
        assert_eq!(rtk.base_station_longitude, Some(-88.20111126338399));
        assert_eq!(rtk.base_station_altitude, Some(189.7029));
        assert_eq!(rtk.std_latitude, 0.013500928);
        assert_eq!(rtk.std_longitude, 0.010167635);
        assert_eq!(rtk.std_altitude, 0.025224999);
        assert_eq!(rtk.antenna1_satellites.gps, Some(10));
        assert_eq!(rtk.antenna1_satellites.beidou, Some(8));
        assert_eq!(rtk.antenna1_satellites.glonass, Some(7));
        assert_eq!(rtk.antenna1_satellites.galileo, Some(7));
        assert_eq!(rtk.base_station_satellites.gps, Some(10));
    }

    #[test]
    fn single_solution_without_a_base_station() {
        let rtk = read(concat!(
            "0010150f11150d11010101252b66e203e042405a2d7dfe7f4556c0893ad942000000000000000000",
            "00000000000000000000002b97ae4332010000000000000000e8e4863fc9de5c3f26180d40151501",
        ))
        .unwrap();

        assert_eq!(rtk.solution, RtkSolution::Single);
        assert_eq!(rtk.base_station_latitude, None);
        assert_eq!(rtk.base_station_altitude, None);
        assert_eq!(rtk.base_station_satellites.gps, Some(0));
    }

    #[test]
    fn qzss_satellites() {
        let mut hex = "00".repeat(80);
        hex.push_str("050301");

        let rtk = read(&hex).unwrap();
        assert_eq!(rtk.antenna1_satellites.qzss, Some(2));
        assert_eq!(rtk.antenna2_satellites.qzss, Some(1));
        assert_eq!(rtk.base_station_satellites.qzss, Some(0));
    }

    #[test]
    fn qzss_satellites_are_unknown_in_the_records_without_them() {
        let rtk = read(&"00".repeat(80)).unwrap();
        assert_eq!(rtk.antenna1_satellites.qzss, None);
    }
}
