use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::rtk_difference::{RTKDifference, RtkSolution};

/// RTK positioning, only on drones with RTK.
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameRTK {
    /// Solution of the RTK positioning: `Fixed` is precise to a few centimeters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub solution: Option<RtkSolution>,
    /// Standard deviation of the latitude, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub std_latitude: Option<f32>,
    /// Standard deviation of the longitude, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub std_longitude: Option<f32>,
    /// Standard deviation of the altitude, in meters
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub std_altitude: Option<f32>,
    /// Number of satellites of the main antenna, all constellations
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub satellites: Option<u8>,
}

impl FrameRTK {
    pub(crate) fn update(&mut self, rtk: &RTKDifference) {
        let satellites = &rtk.antenna1_satellites;
        self.solution = Some(rtk.solution);
        self.std_latitude = Some(rtk.std_latitude);
        self.std_longitude = Some(rtk.std_longitude);
        self.std_altitude = Some(rtk.std_altitude);
        self.satellites = [
            satellites.gps,
            satellites.beidou,
            satellites.glonass,
            satellites.galileo,
        ]
        .into_iter()
        .flatten()
        .reduce(u8::saturating_add);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::BinRead;

    #[test]
    fn satellites_of_all_constellations_are_added() {
        let hex = concat!(
            "003215110f15110f15090b534edae4920a4440fd4684f3e00c56c0fd8a3f432ba90072970a4440db",
            "c5c601df0c56c0f1b33d43c5dccf423201cccccc3f00000000ff325d3c2796263ca8a4ce3c0f0d07",
        );
        let mut data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        data.resize(data.len() + 64, 0);
        let rtk = RTKDifference::read_le(&mut Cursor::new(data)).unwrap();

        let mut frame = FrameRTK::default();
        frame.update(&rtk);

        assert_eq!(frame.solution, Some(RtkSolution::Fixed));
        assert_eq!(frame.std_altitude, Some(0.025224999));
        // 10 GPS, 8 BeiDou, 7 GLONASS and 7 Galileo satellites
        assert_eq!(frame.satellites, Some(32));
    }
}
