use binrw::io::{Read, Seek};
use binrw::{binread, BinRead, BinResult, Endian};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// Manned aircraft received by ADS-B around the drone.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { length: u16 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct ADSBFlightData {
    #[br(temp)]
    flags: u8,
    #[br(temp)]
    #[br(assert(
        HEADER_LENGTH + usize::from(count) * entry_length(flags) <= usize::from(length),
        "too many ADS-B aircraft for the record length"
    ))]
    count: u8,

    /// Older entries can follow, they are not read
    #[br(count = count, args { inner: (flags & EXTENDED_ENTRIES != 0,) })]
    pub aircraft: Vec<ADSBAircraft>,
}

const HEADER_LENGTH: usize = 2;
/// Flag of the entries of 28 bytes, with the altitude above sea level
const EXTENDED_ENTRIES: u8 = 0x40;

fn entry_length(flags: u8) -> usize {
    if flags & EXTENDED_ENTRIES != 0 {
        28
    } else {
        26
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct ADSBAircraft {
    /// ICAO address, 6 hexadecimal characters
    pub icao: String,
    /// degrees
    pub latitude: f64,
    /// degrees
    pub longitude: f64,
    /// meters above sea level, not reported by every drone
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub altitude: Option<i16>,
    /// meters, relative to the drone
    pub relative_altitude: i16,
    /// degrees
    pub heading: f32,
    /// m/s, not reported by every drone
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub speed: Option<u16>,
    /// meters, horizontal distance to the drone
    pub distance: u16,
}

/// Entry of 26 bytes
#[binread]
#[br(little)]
struct Entry {
    icao: [u8; 6],
    #[br(temp)]
    _unknown: u16,
    latitude: i32,
    longitude: i32,
    heading: u16,
    relative_altitude: i16,
    speed: u16,
    #[br(temp)]
    _unknown2: i16,
    distance: u16,
}

/// Entry of 28 bytes
#[binread]
#[br(little)]
struct ExtendedEntry {
    icao: [u8; 6],
    #[br(temp)]
    _unknown: u16,
    latitude: i32,
    longitude: i32,
    altitude: i16,
    #[br(temp)]
    _unknown2: [u8; 3],
    heading: u16,
    relative_altitude: i16,
    #[br(temp)]
    _unknown3: u8,
    distance: u16,
}

/// Reads an entry of 28 bytes if `extended`, else of 26 bytes.
impl BinRead for ADSBAircraft {
    type Args<'a> = (bool,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (extended,): Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(if extended {
            let entry = ExtendedEntry::read_options(reader, endian, ())?;
            ADSBAircraft {
                icao: String::from_utf8_lossy(&entry.icao).into_owned(),
                latitude: degrees(entry.latitude),
                longitude: degrees(entry.longitude),
                altitude: Some(entry.altitude),
                relative_altitude: entry.relative_altitude,
                heading: f32::from(entry.heading) / 10.0,
                speed: None,
                distance: entry.distance,
            }
        } else {
            let entry = Entry::read_options(reader, endian, ())?;
            ADSBAircraft {
                icao: String::from_utf8_lossy(&entry.icao).into_owned(),
                latitude: degrees(entry.latitude),
                longitude: degrees(entry.longitude),
                altitude: None,
                relative_altitude: entry.relative_altitude,
                heading: f32::from(entry.heading) / 10.0,
                speed: Some(entry.speed),
                distance: entry.distance,
            }
        })
    }
}

fn degrees(value: i32) -> f64 {
    f64::from(value) / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::args;
    use binrw::io::Cursor;

    fn read(data: &[u8]) -> BinResult<ADSBFlightData> {
        let length = data.len() as u16;
        ADSBFlightData::read_le_args(&mut Cursor::new(data), args! { length })
    }

    #[test]
    fn entries_of_26_bytes() {
        let data = hex(concat!(
            "0303",
            "6131666331620001c25361024345bbf9b1021500330000004f24",
            "6161316563640001911961022bb5bbf9e30466ff2a000000aa2a",
            "61646162656400013da16202e2eabbf9120d6f02430000007f47",
            "0000000000000000000000000000000000000000000000000000",
        ));

        let data = read(&data).unwrap();

        assert_eq!(data.aircraft.len(), 3);
        let aircraft = &data.aircraft[0];
        assert_eq!(aircraft.icao, "a1fc1b");
        assert_eq!(aircraft.latitude, 39.932866);
        assert_eq!(aircraft.longitude, -105.167549);
        assert_eq!(aircraft.altitude, None);
        assert_eq!(aircraft.relative_altitude, 21);
        assert_eq!(aircraft.heading, 68.9);
        assert_eq!(aircraft.speed, Some(51));
        assert_eq!(aircraft.distance, 9295);
        assert_eq!(data.aircraft[1].relative_altitude, -154);
    }

    #[test]
    fn entries_of_28_bytes() {
        let data = hex("4301613465363338000101fb6202fba5bcfab2010000011c00ae00006420");

        let data = read(&data).unwrap();

        let aircraft = &data.aircraft[0];
        assert_eq!(aircraft.icao, "a4e638");
        assert_eq!(aircraft.latitude, 40.041217);
        assert_eq!(aircraft.longitude, -88.300037);
        assert_eq!(aircraft.altitude, Some(434));
        assert_eq!(aircraft.relative_altitude, 174);
        assert_eq!(aircraft.heading, 2.8);
        assert_eq!(aircraft.speed, None);
        assert_eq!(aircraft.distance, 8292);
    }

    #[test]
    fn record_too_short_for_its_aircraft() {
        assert!(read(&hex("030a0000a2a0")).is_err());
    }

    fn hex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }
}
