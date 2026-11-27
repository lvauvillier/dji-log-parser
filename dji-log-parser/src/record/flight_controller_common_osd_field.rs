use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// State of the flight controller in recent logs, by subtype.
#[binread]
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[br(little, import { length: u16 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum FlightControllerCommonOSDField {
    #[br(magic = 0u16)]
    BatteryOsd(BatteryOsd),
    /// Subtype 1 counts down from 9 to 0, what it counts is unknown
    #[serde(rename_all = "camelCase")]
    Unknown {
        subtype: u16,
        #[br(count = length.saturating_sub(2))]
        data: Vec<u8>,
    },
}

/// Flight time left on the battery and what returning home or landing needs, like the first
/// values of the `SmartBattery` records of older logs.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct BatteryOsd {
    /// seconds, 0 on the ground
    pub useful_time: u16,
    /// seconds
    #[br(map = |x: u16| (x != u16::MAX).then_some(x))]
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_time: Option<u16>,
    /// seconds
    pub land_time: u16,
    /// percent of battery needed to return home
    pub go_home_battery: u8,
    /// percent of battery needed to land
    pub land_battery: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::args;
    use binrw::io::Cursor;
    use binrw::BinRead;

    fn read(hex: &str) -> FlightControllerCommonOSDField {
        let data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let length = data.len() as u16;
        FlightControllerCommonOSDField::read_args(&mut Cursor::new(data), args! { length }).unwrap()
    }

    #[test]
    fn battery() {
        let FlightControllerCommonOSDField::BatteryOsd(battery) =
            read("0000e5052e001e001409000000000000")
        else {
            panic!("not battery information");
        };

        assert_eq!(battery.useful_time, 1509);
        assert_eq!(battery.go_home_time, Some(46));
        assert_eq!(battery.land_time, 30);
        assert_eq!(battery.go_home_battery, 20);
        assert_eq!(battery.land_battery, 9);
    }

    #[test]
    fn go_home_time_is_not_available_before_the_takeoff() {
        let FlightControllerCommonOSDField::BatteryOsd(battery) =
            read("00000000ffff00000000000000000000")
        else {
            panic!("not battery information");
        };

        assert_eq!(battery.go_home_time, None);
    }

    #[test]
    fn other_subtypes_are_kept() {
        let field = read("01000000000901000000000000000000");

        assert!(matches!(
            field,
            FlightControllerCommonOSDField::Unknown { subtype: 1, ref data } if data.len() == 14 && data[3] == 9
        ));
    }
}
