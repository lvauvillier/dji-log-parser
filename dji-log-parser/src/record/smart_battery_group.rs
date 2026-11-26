use binrw::io::{Read, Seek};
use binrw::{binread, BinRead, BinResult, Endian};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum SmartBatteryGroup {
    SmartBatteryStatic(SmartBatteryStatic),
    SmartBatteryDynamic(SmartBatteryDynamic),
    SmartBatterySingleVoltage(SmartBatterySingleVoltage),
}

/// Reads the group from its version and its data length.
impl BinRead for SmartBatteryGroup {
    type Args<'a> = (u8, u16);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(match Layout::read_options(reader, endian, args)? {
            Layout::Static(info) => Self::SmartBatteryStatic(info),
            Layout::Dynamic(info) | Layout::ShiftedDynamic(info) | Layout::LegacyDynamic(info) => {
                Self::SmartBatteryDynamic(info)
            }
            Layout::SingleVoltage(info, _)
            | Layout::ShiftedSingleVoltage(info, _)
            | Layout::LegacySingleVoltage(info) => Self::SmartBatterySingleVoltage(info),
        })
    }
}

/// Records start with the kind of battery information they hold, except in logs written by
/// apps of 2017, which only record dynamic information and cell voltages, starting directly with
/// the battery index and followed by an unused byte.
///
/// From version 14, the index can be preceded by an unknown byte, depending on the aircraft and
/// even from one record to the next.
///
/// The variants are tried in order, until the values read are those of a battery: a legacy
/// record starting with an index of 1 or 2 is told apart from a current one by its length and its
/// values, and a record with the unknown byte by its values and, for cell voltages, by the zeros
/// that follow them.
#[binread]
#[br(little, import(version: u8, length: u16))]
enum Layout {
    #[br(magic = 1u8, pre_assert(length >= 41))]
    Static(#[br(args { version })] SmartBatteryStatic),
    #[br(magic = 2u8, assert(self_0.is_plausible()))]
    Dynamic(SmartBatteryDynamic),
    #[br(magic = 2u8, pre_assert(version >= 14), assert(self_0.is_plausible()))]
    ShiftedDynamic(#[br(pad_before = 1)] SmartBatteryDynamic),
    #[br(magic = 3u8, assert(self_0.length() <= length && self_0.is_plausible()))]
    SingleVoltage(SmartBatterySingleVoltage, #[br(parse_with = zeros)] ()),
    #[br(magic = 3u8, pre_assert(version >= 14))]
    #[br(assert(self_0.length() < length && self_0.is_plausible()))]
    ShiftedSingleVoltage(
        #[br(pad_before = 1)] SmartBatterySingleVoltage,
        #[br(parse_with = zeros)] (),
    ),
    #[br(pre_assert(length == 35))]
    LegacyDynamic(SmartBatteryDynamic),
    #[br(assert(self_0.length() == length && self_0.is_plausible()))]
    LegacySingleVoltage(SmartBatterySingleVoltage),
}

/// Checks that the rest of the record only holds zeros.
#[binrw::parser(reader)]
fn zeros() -> BinResult<()> {
    let position = reader.stream_position()?;
    let mut rest = Vec::new();
    reader.read_to_end(&mut rest)?;
    if rest.iter().all(|byte| *byte == 0) {
        Ok(())
    } else {
        Err(binrw::Error::AssertFail {
            pos: position,
            message: "the record doesn't end with zeros".to_string(),
        })
    }
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatteryStatic {
    #[br(temp, if(version >= 14))]
    _unknown0: u8,
    pub index: u8,
    pub designed_capacity: u32,
    pub loop_times: u16,
    pub full_voltage: u32,
    #[br(temp)]
    _unknown: u16,
    serial_number: u16,
    #[br(temp)]
    _unknown2: [u8; 10],
    #[br(temp)]
    _unknown3: [u8; 5],
    pub version_number: [u8; 8],
    pub battery_life: u8,
    pub battery_type: u8,
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatteryDynamic {
    pub index: u8,
    /// volts
    #[br(map = |x: i32| x as f32 / 1000.0)]
    pub current_voltage: f32,
    // ampere
    #[br(map = |x: i32| x.abs() as f32 / 1000.0)]
    pub current_current: f32,
    /// mAh
    pub full_capacity: u32,
    /// mAh
    pub remained_capacity: u32,
    /// degrees
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub temperature: f32,
    pub cell_count: u8,
    pub capacity_percent: u8,
    pub battery_state: u64,
}

impl SmartBatteryDynamic {
    /// Whether the index and the voltage are those of a battery: they are not when the record is
    /// read with a layout shifted by one byte.
    fn is_plausible(&self) -> bool {
        self.index <= 6 && (0.0..100.0).contains(&self.current_voltage)
    }
}

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct SmartBatterySingleVoltage {
    pub index: u8,
    pub cell_count: u8,
    #[br(count = cell_count, map = |xs: Vec<u16>| xs.into_iter().map(|x| x as f32 / 1000.0).collect())]
    pub cell_voltages: Vec<f32>,
}

impl SmartBatterySingleVoltage {
    /// Length of the record: the index, the cell count, the cell voltages and one more byte, the
    /// kind of information or, in legacy records, the unused byte.
    fn length(&self) -> u16 {
        3 + 2 * u16::from(self.cell_count)
    }

    /// Whether the voltages are those of battery cells: they are not when the record is read with
    /// a layout shifted by one byte.
    fn is_plausible(&self) -> bool {
        self.cell_voltages.iter().all(|voltage| *voltage < 5.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;

    fn read(version: u8, data: &[u8]) -> SmartBatteryGroup {
        let length = data.len() as u16;
        SmartBatteryGroup::read_le_args(&mut Cursor::new(data), (version, length)).unwrap()
    }

    fn dynamic(version: u8, data: &[u8]) -> SmartBatteryDynamic {
        match read(version, data) {
            SmartBatteryGroup::SmartBatteryDynamic(info) => info,
            group => panic!("not dynamic information: {group:?}"),
        }
    }

    fn single_voltage(version: u8, data: &[u8]) -> SmartBatterySingleVoltage {
        match read(version, data) {
            SmartBatteryGroup::SmartBatterySingleVoltage(info) => info,
            group => panic!("not cell voltages: {group:?}"),
        }
    }

    /// Records of the second battery of an Inspire 2, in 2018 and in 2017.
    #[test]
    fn legacy_records_are_told_apart_from_current_ones() {
        let current = dynamic(
            8,
            &[
                0x02, 0x01, 0x32, 0x61, 0x00, 0x00, 0x34, 0xfe, 0xff, 0xff, 0x8d, 0x10, 0x00, 0x00,
                0x11, 0x0e, 0x00, 0x00, 0xe0, 0x00, 0x06, 0x55, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x34, 0x01, 0x64, 0x00, 0x00,
            ],
        );
        assert_eq!((current.index, current.current_voltage), (1, 24.882));
        assert_eq!((current.full_capacity, current.cell_count), (4237, 6));

        let legacy = dynamic(
            8,
            &[
                0x02, 0xbe, 0x62, 0x00, 0x00, 0x06, 0xfd, 0xff, 0xff, 0xb1, 0x10, 0x00, 0x00, 0x30,
                0x0f, 0x00, 0x00, 0x54, 0x01, 0x06, 0x5b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x34, 0x01, 0x64, 0x00, 0x00, 0x01,
            ],
        );
        assert_eq!((legacy.index, legacy.current_voltage), (2, 25.278));
        assert_eq!((legacy.full_capacity, legacy.cell_count), (4273, 6));

        let legacy = single_voltage(
            8,
            &[
                0x02, 0x06, 0x69, 0x10, 0x77, 0x10, 0x78, 0x10, 0x64, 0x10, 0x7d, 0x10, 0x75, 0x10,
                0x40,
            ],
        );
        assert_eq!(legacy.index, 2);
        assert_eq!(
            legacy.cell_voltages,
            [4.201, 4.215, 4.216, 4.196, 4.221, 4.213]
        );
    }

    #[test]
    fn legacy_record_of_the_first_battery_is_not_static_information() {
        let legacy = single_voltage(
            8,
            &[
                0x01, 0x06, 0x66, 0x10, 0x70, 0x10, 0x79, 0x10, 0x79, 0x10, 0x80, 0x10, 0x6d, 0x10,
                0x00,
            ],
        );
        assert_eq!(legacy.index, 1);
        assert_eq!(legacy.cell_count, 6);
    }

    /// Records of a Matrice 30 and a Mavic 3 Enterprise, with an unknown byte before the index.
    #[test]
    fn records_with_a_byte_before_the_index_are_read() {
        let shifted = dynamic(
            14,
            &[
                0x02, 0x00, 0x00, 0x0e, 0x61, 0x00, 0x00, 0x8a, 0xf7, 0xff, 0xff, 0x48, 0x24, 0x00,
                0x00, 0xc5, 0x21, 0x00, 0x00, 0xf0, 0x00, 0x06, 0x63, 0x00, 0x00, 0x00, 0xb8, 0x03,
                0x00, 0x00, 0x00, 0x13, 0x02, 0x64, 0x00, 0x00, 0x08, 0x87, 0x00, 0x00, 0x00, 0x00,
                0x0f, 0x89, 0x0f, 0x09, 0x00, 0x00,
            ],
        );
        assert_eq!((shifted.index, shifted.current_voltage), (0, 24.846));
        assert_eq!((shifted.full_capacity, shifted.cell_count), (9288, 6));

        let shifted = single_voltage(
            14,
            &[
                0x03, 0x00, 0x01, 0x06, 0x2d, 0x10, 0x2e, 0x10, 0x2f, 0x10, 0x29, 0x10, 0x2b, 0x10,
                0x29, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ],
        );
        assert_eq!(shifted.index, 1);
        assert_eq!(
            shifted.cell_voltages,
            [4.141, 4.142, 4.143, 4.137, 4.139, 4.137]
        );

        let current = single_voltage(
            14,
            &[
                0x03, 0x00, 0x04, 0x0b, 0x10, 0x0c, 0x10, 0x0e, 0x10, 0x0a, 0x10, 0x00, 0x00, 0x00,
                0x00, 0x00,
            ],
        );
        let shifted = single_voltage(
            14,
            &[
                0x03, 0x00, 0x00, 0x04, 0xab, 0x0f, 0xa7, 0x0f, 0xae, 0x0f, 0xa5, 0x0f, 0x00, 0x00,
                0x00, 0x00,
            ],
        );
        assert_eq!((current.index, current.cell_count), (0, 4));
        assert_eq!((shifted.index, shifted.cell_voltages[0]), (0, 4.011));
    }
}
