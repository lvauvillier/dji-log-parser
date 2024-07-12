use binrw::binread;
use serde::Serialize;
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::decoder::record_decoder;
use crate::layout::details::ProductType;
use crate::utils;
use crate::Keychain;

pub mod app_gps;
pub mod app_serious_warn;
pub mod app_tip;
pub mod app_warn;
pub mod camera;
pub mod center_battery;
pub mod component_serial;
pub mod custom;
pub mod deform;
pub mod firmware;
pub mod gimbal;
pub mod home;
pub mod key_storage;
pub mod mc_param;
pub mod ofdm;
pub mod osd;
pub mod rc;
pub mod rc_display_field;
pub mod rc_gps;
pub mod recover;
pub mod smart_battery;
pub mod smart_battery_group;
pub mod virtual_stick;

use app_gps::AppGPS;
use app_serious_warn::AppSeriousWarn;
use app_tip::AppTip;
use app_warn::AppWarn;
use camera::Camera;
use center_battery::CenterBattery;
use component_serial::ComponentSerial;
use custom::Custom;
use deform::Deform;
use firmware::Firmware;
use gimbal::Gimbal;
use home::Home;
use key_storage::KeyStorage;
use mc_param::MCParams;
use ofdm::OFDM;
use osd::OSD;
use rc::RC;
use rc_display_field::RCDisplayField;
use rc_gps::RCGPS;
use recover::Recover;
use smart_battery::SmartBattery;
use smart_battery_group::*;
use virtual_stick::VirtualStick;

const END_BYTE: u8 = 0xFF;

/// Represents the different types of records.
///
/// Each variant of this enum corresponds to a specific type of record in the log file.
/// Records typically consist of a 'magic' byte indicating the record type, followed by the length of the record,
/// the actual data, and then a terminating byte of value `0xff`.
///
#[binread]
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "content")]
#[br(little, import { version: u8, keychain: &RefCell<Keychain>, product_type: ProductType = ProductType::None })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum Record {
    #[br(magic = 1u8)]
    OSD(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 1, version, keychain, self_0),
            args { version }
        )]
        OSD,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 2u8)]
    Home(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 2, version, keychain, self_0),
            args { version }
        )]
        Home,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 3u8)]
    Gimbal(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 3, version, keychain, self_0),
            args { version }
        )]
        Gimbal,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 4u8)]
    RC(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 4, version, keychain, self_0),
            args { product_type, version }
        )]
        RC,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 5u8)]
    Custom(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 5, version, keychain, self_0)
        )]
        Custom,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 6u8)]
    Deform(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 6, version, keychain, self_0)
        )]
        Deform,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 7u8)]
    CenterBattery(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 7, version, keychain, self_0),
            args { version }
        )]
        CenterBattery,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 8u8)]
    SmartBattery(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 8, version, keychain, self_0)
        )]
        SmartBattery,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 9u8)]
    AppTip(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 9, version, keychain, self_0),
            args { length: self_0 }
        )]
        AppTip,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 10u8)]
    AppWarn(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 10, version, keychain, self_0),
            args { length: self_0 }
        )]
        AppWarn,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 11u8)]
    RCGPS(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 11, version, keychain, self_0)
        )]
        RCGPS,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 13u8)]
    Recover(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 13, version, keychain, self_0),
            args { version }
        )]
        Recover,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 14u8)]
    AppGPS(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 14, version, keychain, self_0)
        )]
        AppGPS,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 15u8)]
    Firmware(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 15, version, keychain, self_0)
        )]
        Firmware,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 19u8)]
    MCParams(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 19, version, keychain, self_0)
        )]
        MCParams,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 22u8)]
    SmartBatteryGroup(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 22, version, keychain, self_0)
        )]
        SmartBatteryGroup,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 24u8)]
    AppSeriousWarn(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 24, version, keychain, self_0),
            args { length: self_0 }
        )]
        AppSeriousWarn,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 25u8)]
    Camera(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 25, version, keychain, self_0),
        )]
        Camera,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 33u8)]
    VirtualStick(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 33, version, keychain, self_0),
        )]
        VirtualStick,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 40u8)]
    ComponentSerial(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 40, version, keychain, self_0),
        )]
        ComponentSerial,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 49u8)]
    OFDM(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 49, version, keychain, self_0),
        )]
        OFDM,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 50u8)]
    KeyStorageRecover(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(count = self_0)] Vec<u8>,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 56u8)]
    KeyStorage(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 56, version, keychain, self_0)
        )]
        KeyStorage,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    #[br(magic = 62u8)]
    RCDisplayField(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 62, version, keychain, self_0),
        )]
        RCDisplayField,
        #[br(temp, assert(self_2 == END_BYTE))] u8,
    ),
    JPEG(#[br(parse_with = utils::read_jpeg)] Vec<u8>),
    // Valid record of unknown data
    Unknown(
        u8, // record_type
        #[br(temp, args(version <= 12), parse_with = utils::read_u16, assert(self_1 > 2))] u16,
        #[br(
            pad_size_to = self_1,
            count = if version <= 6 {
                self_1
            } else {
                self_1 - 2
            },
            map_stream = |reader| record_decoder(reader, self_0, version, keychain, self_1)
        )]
        Vec<u8>,
        #[br(temp, assert(self_3 == END_BYTE))] u8,
    ),
    // Invalid data, try to seek to next record
    Invalid(#[br(parse_with = utils::seek_to_next_record, assert(!self_0.is_empty()))] Vec<u8>),
}
