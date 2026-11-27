use binrw::io::{Read, Seek};
use binrw::{BinRead, BinResult, Endian};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

/// Firmware versions of the components of the aircraft, from version 8. The records of older
/// logs have another content.
///
/// The numbers are in the order of the records. In the version 8 logs of DJI GO, they seem
/// reversed (3.3.0.21 is recorded as 21.0.3.3), but not in the logs of the Mavic 2 Enterprise.
/// In version 9, the components have the same version: the one of the firmware of the aircraft.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Firmware {
    pub versions: Vec<FirmwareVersion>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FirmwareVersion {
    pub sender_type: SenderType,
    /// Index of the component among those of its type, such as the battery 0 or 1
    pub index: u8,
    /// Four numbers, such as "9.3.0.39"
    pub version: String,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum SenderType {
    Camera,
    MC,
    Gimbal,
    RC,
    Battery,
    #[serde(untagged)]
    Unknown(u8),
}

impl SenderType {
    pub fn from(value: u8) -> Self {
        match value {
            1 => SenderType::Camera,
            3 => SenderType::MC,
            4 => SenderType::Gimbal,
            6 => SenderType::RC,
            11 => SenderType::Battery,
            _ => SenderType::Unknown(value),
        }
    }
}

const ENTRY_LENGTH: usize = 6;

/// Reads entries of 6 bytes: the sender type, the index and the four numbers of the version, up
/// to the length of the record or to an entry of zeros.
impl BinRead for Firmware {
    type Args<'a> = (u8, u16);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        (version, length): Self::Args<'_>,
    ) -> BinResult<Self> {
        let position = reader.stream_position()?;
        if version < 8 {
            return Err(binrw::Error::AssertFail {
                pos: position,
                message: "firmware record before version 8".to_owned(),
            });
        }
        let mut data = vec![0; length.into()];
        reader.read_exact(&mut data)?;

        let (entries, _) = data.as_chunks::<ENTRY_LENGTH>();
        let versions = entries
            .iter()
            .take_while(|entry| entry.iter().any(|byte| *byte != 0))
            .map(|entry| {
                let [sender_type, index, numbers @ ..] = *entry;
                FirmwareVersion {
                    sender_type: SenderType::from(sender_type),
                    index,
                    version: numbers.map(|number| number.to_string()).join("."),
                }
            })
            .collect();
        Ok(Firmware { versions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;

    #[test]
    fn versions_up_to_the_padding() {
        let data = [
            0x03, 0x00, 0x05, 0x05, 0x00, 0x21, 0x06, 0x00, 0x01, 0x02, 0x04, 0x2d, 0x0b, 0x01,
            0x09, 0x03, 0x00, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let firmware =
            Firmware::read_le_args(&mut Cursor::new(data), (14, data.len() as u16)).unwrap();

        assert_eq!(firmware.versions.len(), 3);
        assert_eq!(firmware.versions[0].sender_type, SenderType::MC);
        assert_eq!(firmware.versions[0].version, "5.5.0.33");
        assert_eq!(firmware.versions[2].sender_type, SenderType::Battery);
        assert_eq!(firmware.versions[2].index, 1);
    }

    #[test]
    fn old_records_are_rejected() {
        let data = [0x7e, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(Firmware::read_le_args(&mut Cursor::new(data), (6, 6)).is_err());
    }
}
