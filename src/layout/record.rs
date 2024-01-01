use std::cell::RefCell;
use std::f64::consts::PI;

use binrw::helpers::until;
use binrw::io::NoSeek;
use binrw::{binread, parser, BinResult};

use crate::decoder::record_decoder;
use crate::layout::feature_point::FeaturePoint;
use crate::Keychain;

/// Represents the different types of records.
///
/// Each variant of this enum corresponds to a specific type of record in the log file.
/// Records typically consist of a 'magic' byte indicating the record type, followed by the length of the record,
/// the actual data, and then a terminating byte of value `0xff`.
///
#[binread]
#[derive(Debug)]
#[br(little, import(version: u8, keychain: &RefCell<Keychain>))]
pub enum Record {
    #[br(magic = 1u8)]
    OSD(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 1, version, keychain, self_0)) )]
        OSD,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 56u8)]
    KeyStorage(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_0, map_stream = |reader| NoSeek::new(record_decoder(reader, 56, version, keychain, self_0)))]
         KeyStorage,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 50u8)]
    Recover(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(count = self_0)] Vec<u8>,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    // Valid record of unknown data
    Unknown(
        u8, // record_type
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_1,
            count = if version <= 6 {
                self_1
            } else {
                self_1 - 2
            },
            map_stream = |reader| NoSeek::new(record_decoder(reader, self_0, version, keychain, self_1)))]
        Vec<u8>,
        #[br(temp, assert(self_3 == 0xff))] u8,
    ),
    // Invalid Record, parse util we get a terminating byte of value `0xff`
    Invalid(#[br(parse_with = until(|&byte| byte == 0xff))] Vec<u8>),
}

/// Reads a 16-bit unsigned integer from a given reader.
///
/// This function attempts to read 2 bytes from the specified reader and convert them into a `u16`. It can adjust its behavior based on the `from_u8` flag.
///
/// # Parameters
/// - `from_u8`: A boolean flag indicating how to read the bytes.
///   - If `true`, only one byte is read from the reader, and it's treated as the lower part of the `u16`.
///   - If `false`, two bytes are read and directly converted to `u16`.
///
/// # Returns
/// This function returns a `BinResult<u16>`. On successful read and conversion, it returns `Ok(u16)`.
/// If there is any error in reading from the reader, it returns an error encapsulated in the `BinResult`.
///
/// # Errors
/// This function will return an error if the reader fails to provide the necessary number of bytes (1 or 2, based on `from_u8`).
/// ```
#[parser(reader)]
pub fn read_u16(from_u8: bool) -> BinResult<u16> {
    let mut bytes = [0; 2];
    reader.read_exact(&mut bytes[if from_u8 { 0..=0 } else { 0..=1 }])?;
    Ok(u16::from_le_bytes(bytes))
}

fn sub_byte_field(byte: u8, mask: u8) -> u8 {
    // First, use "mask" to select the bits that we want, and move them to the low-order part of the byte:
    let mut byte = byte;
    byte &= mask;

    let mut mask = mask;
    while mask != 0x00 && (mask & 0x01) == 0 {
        byte >>= 1;
        mask >>= 1;
    }
    byte
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct KeyStorage {
    pub feature_point: FeaturePoint,
    #[br(temp)]
    data_length: u16,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct OSD {
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub longitude: f64,
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub lattitude: f64,

    pub barometer_height: i16,
    pub speed_x: i16,
    pub speed_y: i16,
    pub speed_z: i16,
    pub pitch: i16,
    pub roll: i16,
    pub yaw: i16,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x7F)))]
    pub control_mode: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x80)))]
    pub rc_outcontrol: u8,

    pub app_command: u8,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x01)))]
    pub can_ioc_work: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x06)))]
    pub ground_or_sky: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x08)))]
    pub is_motor_up: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x10)))]
    pub is_swave_work: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0xE0)))]
    pub go_home_status: u8,

    #[br(temp)]
    _bitpack3: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x01)))]
    pub is_vision_used: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x06)))]
    pub voltage_warning: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x10)))]
    pub is_imu_preheated: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x60)))]
    pub mode_channel: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x80)))]
    pub is_gps_valid: u8,

    #[br(temp)]
    _bitpack4: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x01)))]
    pub is_compass_error: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x02)))]
    pub wave_error: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x3C)))]
    pub gps_level: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0xC0)))]
    pub battery_type: u8,

    #[br(temp)]
    _bitpack5: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x01)))]
    pub is_out_of_limit: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x02)))]
    pub is_go_home_height_modified: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x04)))]
    pub is_propeller_catapult: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x08)))]
    pub is_motor_blocked: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x10)))]
    pub is_not_enough_force: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x20)))]
    pub is_barometer_dead_in_air: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x40)))]
    pub is_vibrating: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x80)))]
    pub is_acceletor_over_range: u8,

    pub gps_num: u8,
    pub flight_action: u8,
    pub motor_start_failed_cause: u8,

    #[br(temp)]
    _bitpack6: u8,
    #[br(calc(sub_byte_field(_bitpack6, 0x0F)))]
    pub non_gpscause: u8,
    #[br(calc(sub_byte_field(_bitpack6, 0x10)))]
    pub waypoint_limit_mode: u8,

    pub battery: u8,
    pub sWaveHeight: u8,
    pub flyTime: u16,
    pub motorRevolution: u8,
    #[br(temp)]
    _unknown: u16,
    pub version: u8,
    pub droneType: u8,
}
