use binrw::binread;
use chrono::{DateTime, Utc};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::protobuf::{fields, Value};
use crate::utils::c_string;

/// Information about the aircraft, written once per flight in recent logs: a header, then a
/// message encoded as protocol buffers.
///
/// The header holds the module and the key of the message (`u16` each), the length of the message
/// (`u16`), a zero and flags whose high bit tells that a timestamp follows, always set in these
/// records. The `FlightRecordTypeFrameData` records (type 254) share this header.
#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct AircraftInfo {
    /// Low byte of the module, which tells the type of the message
    #[br(temp)]
    message_type: u8,
    /// High byte of the module (0), then the key (always 1)
    #[br(temp)]
    _unknown: [u8; 3],
    /// Length of the message, then the zero and the flags in the high byte
    #[br(temp)]
    message_length: u32,
    #[br(map = |x: i64| {
        DateTime::from_timestamp(x / 1000, (x % 1000 * 1_000_000) as u32).unwrap_or_default()
    })]
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    pub timestamp: DateTime<Utc>,
    #[br(temp)]
    #[br(count = message_length & 0x7FFF_FFFF)]
    message: Vec<u8>,

    #[br(calc = AircraftInfoMessage::new(message_type, message))]
    pub message: AircraftInfoMessage,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum AircraftInfoMessage {
    SerialNumbers(SerialNumbers),
    #[serde(rename_all = "camelCase")]
    FlightTime {
        /// Total flight time before this flight, in seconds: of the aircraft, or maybe of the remote
        /// controller or the app
        total_flight_time: f64,
    },
    #[serde(rename_all = "camelCase")]
    Unknown {
        message_type: u8,
        data: Vec<u8>,
    },
}

/// Serial numbers of the aircraft components, not reported by every aircraft
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SerialNumbers {
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub camera_sn: Option<String>,
    pub battery_sns: Vec<String>,
    /// Serial numbers of unknown components, by field number
    pub other_sns: Vec<(u32, String)>,
}

impl AircraftInfoMessage {
    const FLIGHT_TIME: u8 = 5;
    const SERIAL_NUMBERS: u8 = 7;

    fn new(message_type: u8, data: Vec<u8>) -> Self {
        let message = match message_type {
            Self::FLIGHT_TIME => Self::flight_time(&data),
            Self::SERIAL_NUMBERS => Self::serial_numbers(&data),
            _ => None,
        };
        message.unwrap_or(Self::Unknown { message_type, data })
    }

    fn flight_time(data: &[u8]) -> Option<Self> {
        let mut total_flight_time = None;
        for field in fields(data)? {
            if let (1, Value::Fixed64(value)) = field {
                total_flight_time = Some(f64::from_le_bytes(value));
            }
        }
        total_flight_time.map(|total_flight_time| Self::FlightTime { total_flight_time })
    }

    fn serial_numbers(data: &[u8]) -> Option<Self> {
        let mut serial_numbers = SerialNumbers::default();
        for field in fields(data)? {
            let (number, Value::Bytes(bytes)) = field else {
                return None;
            };
            let serial_number = c_string(bytes.to_vec());
            match number {
                1 => serial_numbers.camera_sn = Some(serial_number),
                7 | 8 => serial_numbers.battery_sns.push(serial_number),
                _ => serial_numbers.other_sns.push((number, serial_number)),
            }
        }
        Some(Self::SerialNumbers(serial_numbers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::BinRead;

    fn read(hex: &str) -> AircraftInfo {
        let data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        AircraftInfo::read(&mut Cursor::new(data)).unwrap()
    }

    #[test]
    fn serial_numbers() {
        let info = read(concat!(
            "070001004200008034914db896010000",
            "3220543630434e4635373844433142333530000000000000000000000000000000",
            "003a0e344255504b434d44413232303732420e344255504b434d44413232303632",
            "0000000000000000000000000000",
        ));

        assert_eq!(info.timestamp.to_rfc3339(), "2025-05-10T03:46:53.364+00:00");
        let AircraftInfoMessage::SerialNumbers(serial_numbers) = info.message else {
            panic!("not serial numbers: {:?}", info.message);
        };
        assert_eq!(serial_numbers.camera_sn, None);
        assert_eq!(
            serial_numbers.battery_sns,
            ["4BUPKCMDA22072", "4BUPKCMDA22062"]
        );
        assert_eq!(
            serial_numbers.other_sns,
            [(6, "T60CNF578DC1B350".to_owned())]
        );
    }

    #[test]
    fn flight_time() {
        let info = read("05000100090000805b78a51b8b0100000900109d9bf0efb54000000000000000");

        assert!(matches!(
            info.message,
            AircraftInfoMessage::FlightTime { total_flight_time } if total_flight_time == 5615.939874473959
        ));
    }

    #[test]
    fn unknown_message() {
        let info = read("0900010001000080e098ecec8a01000000");

        assert!(matches!(
            info.message,
            AircraftInfoMessage::Unknown { message_type: 9, ref data } if data == &[0]
        ));
    }
}
