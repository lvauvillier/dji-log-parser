use binrw::io::{Read, Seek};
use binrw::{BinRead, BinResult, Endian};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::protobuf::{fields, Value};

/// Messages shown by the DJI Fly app, which replace the tips and warnings records of older logs.
///
/// Each record updates some of the parts of the display: the parts that are not updated are
/// unchanged.
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct GOBusiness {
    /// Status bar, such as "In flight", when it changes
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub status: Option<GOBusinessStatus>,
    /// Tips shown for a few seconds, when they change: an empty list removes them
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub tips: Option<Vec<GOBusinessTip>>,
    /// Full list of the warnings, when it changes
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub warnings: Option<Vec<GOBusinessWarning>>,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct GOBusinessStatus {
    pub text: String,
    /// 1 to 3, the higher the more serious
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub level: Option<u8>,
    pub is_blinking: bool,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct GOBusinessTip {
    pub identifier: String,
    pub text: String,
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub level: Option<u8>,
    /// Seconds the tip is shown
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub duration: Option<f64>,
}

#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct GOBusinessWarning {
    /// Text ending with a code, such as "GPS signal weak... (Code: 30049)"
    pub text: String,
    /// 1 to 3, the higher the more serious
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub level: Option<u8>,
}

const STATUS: u32 = 2;
const TIP: u32 = 3;
const WARNING: u32 = 4;
const TIPS_CHANGED: u32 = 5;
const STATUS_CHANGED: u32 = 6;
const WARNINGS_CHANGED: u32 = 7;

impl GOBusiness {
    fn parse(data: &[u8]) -> Option<Self> {
        let mut business = GOBusiness::default();
        let (mut status, mut tips, mut warnings) = (None, Vec::new(), Vec::new());
        for (number, value) in fields(data)? {
            match (number, value) {
                (STATUS, Value::Bytes(bytes)) => status = Some(GOBusinessStatus::parse(bytes)?),
                (TIP, Value::Bytes(bytes)) => tips.push(GOBusinessTip::parse(bytes)?),
                (WARNING, Value::Bytes(bytes)) => warnings.push(GOBusinessWarning::parse(bytes)?),
                (STATUS_CHANGED, Value::Varint(1)) => {
                    business.status = Some(GOBusinessStatus::default())
                }
                (TIPS_CHANGED, Value::Varint(1)) => business.tips = Some(Vec::new()),
                (WARNINGS_CHANGED, Value::Varint(1)) => business.warnings = Some(Vec::new()),
                _ => {}
            }
        }
        // The flags tell which parts are updated, even to become empty
        if business.status.is_some() {
            business.status = Some(status.unwrap_or_default());
        }
        if business.tips.is_some() {
            business.tips = Some(tips);
        }
        if business.warnings.is_some() {
            business.warnings = Some(warnings);
        }
        Some(business)
    }
}

impl GOBusinessStatus {
    fn parse(data: &[u8]) -> Option<Self> {
        let mut status = GOBusinessStatus::default();
        for field in fields(data)? {
            match field {
                (1, Value::Bytes(text)) => status.text = String::from_utf8_lossy(text).into(),
                (2, Value::Varint(blinking)) => status.is_blinking = blinking != 0,
                (3, Value::Varint(level)) => status.level = level.try_into().ok(),
                _ => {}
            }
        }
        Some(status)
    }
}

impl GOBusinessTip {
    fn parse(data: &[u8]) -> Option<Self> {
        let mut tip = GOBusinessTip::default();
        for field in fields(data)? {
            match field {
                (1, Value::Varint(level)) => tip.level = level.try_into().ok(),
                (2, Value::Bytes(identifier)) => {
                    tip.identifier = String::from_utf8_lossy(identifier).into()
                }
                (3, Value::Bytes(text)) => tip.text = String::from_utf8_lossy(text).into(),
                (5, Value::Fixed64(milliseconds)) => {
                    tip.duration = Some(f64::from_le_bytes(milliseconds) / 1000.0)
                }
                _ => {}
            }
        }
        Some(tip)
    }
}

impl GOBusinessWarning {
    fn parse(data: &[u8]) -> Option<Self> {
        let mut warning = GOBusinessWarning::default();
        for field in fields(data)? {
            match field {
                (1, Value::Bytes(text)) => warning.text = String::from_utf8_lossy(text).into(),
                (2, Value::Varint(level)) => warning.level = level.try_into().ok(),
                _ => {}
            }
        }
        Some(warning)
    }
}

/// Reads the protocol buffers message of the record, from its data length.
impl BinRead for GOBusiness {
    type Args<'a> = (u16,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        (length,): Self::Args<'_>,
    ) -> BinResult<Self> {
        let position = reader.stream_position()?;
        let mut data = vec![0; length.into()];
        reader.read_exact(&mut data)?;

        GOBusiness::parse(&data).ok_or_else(|| binrw::Error::AssertFail {
            pos: position,
            message: "invalid DJI Fly messages".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(hex: &str) -> GOBusiness {
        let data: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        GOBusiness::parse(&data).unwrap()
    }

    #[test]
    fn tip() {
        let business = parse(concat!(
            "08011a4c12063132303835321a30486f6d65706f696e7420757064617465642e2052544820616c74",
            "69747564652061646a757374656420746f203530206d211d1ed81ae4be624229000000000070a740",
            "2801",
        ));

        let tips = business.tips.unwrap();
        assert_eq!(tips[0].identifier, "120852");
        assert_eq!(
            tips[0].text,
            "Homepoint updated. RTH altitude adjusted to 50 m"
        );
        assert_eq!(tips[0].duration, Some(3.0));
        assert!(business.status.is_none());
        assert!(business.warnings.is_none());
    }

    #[test]
    fn status() {
        let business = parse("080112130a0f475053207369676e616c207765616b18033001");

        let status = business.status.unwrap();
        assert_eq!(status.text, "GPS signal weak");
        assert_eq!(status.level, Some(3));
    }

    #[test]
    fn full_list_of_warnings() {
        let business = parse(concat!(
            "080122320a2e52656d6f74652049442066756e6374696f6e616c697479206e6f726d616c28436f",
            "64653a2031423038303030332910013801",
        ));

        let warnings = business.warnings.unwrap();
        assert_eq!(
            warnings[0].text,
            "Remote ID functionality normal(Code: 1B080003)"
        );
        assert_eq!(warnings[0].level, Some(1));
    }

    #[test]
    fn emptied_warnings() {
        let business = parse("08013801");

        assert_eq!(business.warnings.unwrap().len(), 0);
    }
}
