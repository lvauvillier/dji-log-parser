use std::cell::RefCell;

use binrw::binread;
use binrw::helpers::until;

use crate::decoder::record_decoder;

use crate::layout::info::ProductType;
use crate::utils;
use crate::Keychain;

pub mod app_gps;
pub mod app_serious_warn;
pub mod app_tip;
pub mod app_warn;
pub mod center_battery;
pub mod custom;
pub mod deform;
pub mod firmware;
pub mod gimbal;
pub mod home;
pub mod key_storage;
pub mod mc_param;
pub mod osd;
pub mod rc;
pub mod rc_gps;
pub mod recover_info;
pub mod smart_battery;
pub mod smart_battery_group;

use app_gps::AppGPS;
use app_serious_warn::AppSeriousWarn;
use app_tip::AppTip;
use app_warn::AppWarn;
use center_battery::CenterBattery;
use custom::Custom;
use deform::Deform;
use firmware::Firmware;
use gimbal::Gimbal;
use home::Home;
use key_storage::KeyStorage;
use mc_param::MCParams;
use osd::OSD;
use rc::RC;
use rc_gps::RCGPS;
use recover_info::RecoverInfo;
use smart_battery::SmartBattery;
use smart_battery_group::SmartBatteryGroup;

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
            map_stream = |reader| record_decoder(reader, 1, version, keychain, self_0)
        )]
        OSD,
        #[br(temp, assert(self_2 == 0xff))] u8,
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
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 3u8)]
    Gimbal(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 3, version, keychain, self_0)
        )]
        Gimbal,
        #[br(temp, assert(self_2 == 0xff))] u8,
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
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 5u8)]
    Custom(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 5, version, keychain, self_0)
        )]
        Custom,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 6u8)]
    Deform(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 6, version, keychain, self_0)
        )]
        Deform,
        #[br(temp, assert(self_2 == 0xff))] u8,
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
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 8u8)]
    SmartBattery(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 8, version, keychain, self_0)
        )]
        SmartBattery,
        #[br(temp, assert(self_2 == 0xff))] u8,
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
        #[br(temp, assert(self_2 == 0xff))] u8,
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
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 11u8)]
    RCGPS(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 11, version, keychain, self_0)
        )]
        RCGPS,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 13u8)]
    RecoverInfo(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 13, version, keychain, self_0),
            args { version }
        )]
        RecoverInfo,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 14u8)]
    AppGPS(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 14, version, keychain, self_0)
        )]
        AppGPS,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 15u8)]
    Firmware(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 15, version, keychain, self_0)
        )]
        Firmware,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 19u8)]
    MCParams(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 19, version, keychain, self_0)
        )]
        MCParams,
        #[br(temp, assert(self_2 == 0xff))] u8,
    ),
    #[br(magic = 22u8)]
    SmartBatteryGroup(
        #[br(temp, args(version <= 12), parse_with = utils::read_u16)] u16,
        #[br(
            pad_size_to = self_0,
            map_stream = |reader| record_decoder(reader, 22, version, keychain, self_0)
        )]
        SmartBatteryGroup,
        #[br(temp, assert(self_2 == 0xff))] u8,
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
            map_stream = |reader| record_decoder(reader, self_0, version, keychain, self_1)
        )]
        Vec<u8>,
        #[br(temp, assert(self_3 == 0xff))] u8,
    ),
    // Invalid Record, parse util we get a terminating byte of value `0xff`
    Invalid(#[br(parse_with = until(|&byte| byte == 0xff))] Vec<u8>),
}
