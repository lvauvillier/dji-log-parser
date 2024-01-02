use std::cell::RefCell;

use binrw::binread;
use binrw::helpers::until;
use binrw::io::NoSeek;

use crate::decoder::record_decoder;

use crate::layout::info::ProductType;
use crate::utils;
use crate::Keychain;

pub mod center_battery;
pub mod custom;
pub mod deform;
pub mod gimbal;
pub mod home;
pub mod key_storage;
pub mod osd;
pub mod rc;
pub mod smart_battery;

use center_battery::CenterBattery;
use custom::Custom;
use deform::Deform;
use gimbal::Gimbal;
use home::Home;
use key_storage::KeyStorage;
use osd::OSD;
use rc::RC;
use smart_battery::SmartBattery;

/// Represents the different types of records.
///
/// Each variant of this enum corresponds to a specific type of record in the log file.
/// Records typically consist of a 'magic' byte indicating the record type, followed by the length of the record,
/// the actual data, and then a terminating byte of value `0xff`.
///
#[binread]
#[derive(Debug)]
#[br(little, import { version: u8, keychain: &RefCell<Keychain>, product_type: ProductType = ProductType::None })]
pub enum Record {
    #[br(magic = 1u8)]
    OSD(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 1, version, keychain, self_0))
        )]
        OSD,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 2u8)]
    Home(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 2, version, keychain, self_0))
        )]
        Home,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 3u8)]
    Gimbal(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 3, version, keychain, self_0))
        )]
        Gimbal,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 4u8)]
    RC(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 4, version, keychain, self_0)),
            args { product_type }
        )]
        RC,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 5u8)]
    Custom(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 5, version, keychain, self_0))
        )]
        Custom,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 6u8)]
    Deform(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 6, version, keychain, self_0))
        )]
        Deform,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 7u8)]
    CenterBattery(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 7, version, keychain, self_0))
        )]
        CenterBattery,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 8u8)]
    SmartBattery(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 8, version, keychain, self_0))
        )]
        SmartBattery,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 56u8)]
    KeyStorage(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| NoSeek::new(record_decoder(reader, 56, version, keychain, self_0))
        )]
        KeyStorage,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 50u8)]
    Recover(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(count = self_0)] Vec<u8>,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    // Valid record of unknown data
    Unknown(
        u8, // record_type
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_1,
            count = if version <= 6 {
                self_1
            } else {
                self_1 - 2
            },
            map_stream = |reader| NoSeek::new(record_decoder(reader, self_0, version, keychain, self_1))
        )]
        Vec<u8>,
        #[br(temp, assert(self_3 == 0xff))] u8,
    ),
    // Invalid Record, parse util we get a terminating byte of value `0xff`
    Invalid(#[br(parse_with = until(|&byte| byte == 0xff))] Vec<u8>),
}
