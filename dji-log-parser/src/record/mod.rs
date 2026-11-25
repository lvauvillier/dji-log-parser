use binrw::io::{Read, Seek, SeekFrom};
use binrw::{args, BinRead, BinResult, Endian};
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
#[derive(Serialize, Debug)]
#[serde(tag = "type", content = "content")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum Record {
    OSD(OSD),
    Home(Home),
    Gimbal(Gimbal),
    RC(RC),
    Custom(Custom),
    Deform(Deform),
    CenterBattery(CenterBattery),
    SmartBattery(SmartBattery),
    AppTip(AppTip),
    AppWarn(AppWarn),
    RCGPS(RCGPS),
    Recover(Recover),
    AppGPS(AppGPS),
    Firmware(Firmware),
    MCParams(MCParams),
    SmartBatteryGroup(SmartBatteryGroup),
    AppSeriousWarn(AppSeriousWarn),
    Camera(Camera),
    VirtualStick(VirtualStick),
    ComponentSerial(ComponentSerial),
    OFDM(OFDM),
    KeyStorageRecover(Vec<u8>),
    KeyStorage(KeyStorage),
    RCDisplayField(RCDisplayField),
    JPEG(Vec<u8>),
    /// Valid record of unknown data: record type and decoded content
    Unknown(u8, Vec<u8>),
    /// Invalid data, skipped until the next record
    Invalid(Vec<u8>),
}

impl Record {
    /// Reads the record at the current position.
    ///
    /// Tries, in order: the record type given by the first byte, a JPEG image, a record of
    /// unknown content, and finally skips invalid data up to the next record. The position is
    /// restored before each attempt.
    ///
    /// `product_type` is needed by some records whose values depend on the aircraft model.
    pub(crate) fn read<R: Read + Seek>(
        reader: &mut R,
        version: u8,
        product_type: ProductType,
        keychain: &RefCell<Keychain>,
    ) -> BinResult<Record> {
        let start = reader.stream_position()?;
        let restart = |reader: &mut R| reader.seek(SeekFrom::Start(start));

        Self::read_known(reader, version, product_type, keychain)
            .or_else(|_| {
                restart(reader)?;
                Self::read_jpeg(reader)
            })
            .or_else(|_| {
                restart(reader)?;
                Self::read_unknown(reader, version, keychain)
            })
            .or_else(|_| {
                restart(reader)?;
                Self::read_invalid(reader)
            })
    }

    /// Reads a record of a known type. Fails for unknown types and invalid records.
    fn read_known<R: Read + Seek>(
        reader: &mut R,
        version: u8,
        product_type: ProductType,
        keychain: &RefCell<Keychain>,
    ) -> BinResult<Record> {
        let mut content = RecordContent::read_header(reader, version, keychain)?;
        let length = content.length;

        let record = match content.record_type {
            1 => Record::OSD(content.read::<OSD>(args! { version })?),
            2 => Record::Home(content.read::<Home>(args! { version })?),
            3 => Record::Gimbal(content.read::<Gimbal>(args! { version })?),
            4 => Record::RC(content.read::<RC>(args! { version, product_type })?),
            5 => Record::Custom(content.read(())?),
            6 => Record::Deform(content.read(())?),
            7 => Record::CenterBattery(content.read::<CenterBattery>(args! { version })?),
            8 => Record::SmartBattery(content.read(())?),
            9 => Record::AppTip(content.read::<AppTip>(args! { length })?),
            10 => Record::AppWarn(content.read::<AppWarn>(args! { length })?),
            11 => Record::RCGPS(content.read(())?),
            13 => Record::Recover(content.read::<Recover>(args! { version })?),
            14 => Record::AppGPS(content.read(())?),
            15 => Record::Firmware(content.read(())?),
            19 => Record::MCParams(content.read(())?),
            22 => Record::SmartBatteryGroup(content.read(())?),
            24 => Record::AppSeriousWarn(content.read::<AppSeriousWarn>(args! { length })?),
            25 => Record::Camera(content.read(())?),
            33 => Record::VirtualStick(content.read(())?),
            40 => Record::ComponentSerial(content.read(())?),
            49 => Record::OFDM(content.read(())?),
            // Not encoded
            50 => Record::KeyStorageRecover(content.read_raw()?),
            56 => Record::KeyStorage(content.read(())?),
            62 => Record::RCDisplayField(content.read(())?),
            // Unknown type: no allocated error message, this is the common case for many logs
            _ => return Err(binrw::Error::NoVariantMatch { pos: content.start }),
        };

        Ok(record)
    }

    fn read_jpeg<R: Read + Seek>(reader: &mut R) -> BinResult<Record> {
        utils::read_jpeg(reader, Endian::Little, ()).map(Record::JPEG)
    }

    fn read_unknown<R: Read + Seek>(
        reader: &mut R,
        version: u8,
        keychain: &RefCell<Keychain>,
    ) -> BinResult<Record> {
        let mut content = RecordContent::read_header(reader, version, keychain)?;
        let length = content.length;
        if length <= 2 {
            return Err(binrw::Error::AssertFail {
                pos: content.start,
                message: "unknown record too short".to_owned(),
            });
        }

        let count = if version <= 6 { length } else { length - 2 };
        let data = content.read::<Vec<u8>>(args! { count: count.into() })?;
        Ok(Record::Unknown(content.record_type, data))
    }

    fn read_invalid<R: Read + Seek>(reader: &mut R) -> BinResult<Record> {
        let pos = reader.stream_position()?;
        let data = utils::seek_to_next_record(reader, Endian::Little, ())?;
        if data.is_empty() {
            return Err(binrw::Error::AssertFail {
                pos,
                message: "no invalid data before next record".to_owned(),
            });
        }
        Ok(Record::Invalid(data))
    }
}

/// A record being read: its header has been parsed, its content is read with [`Self::read`].
struct RecordContent<'r, 'k, R> {
    reader: &'r mut R,
    version: u8,
    keychain: &'k RefCell<Keychain>,
    record_type: u8,
    length: u16,
    /// Position of the content, right after the header
    start: u64,
}

impl<'r, 'k, R: Read + Seek> RecordContent<'r, 'k, R> {
    /// Reads the record type and content length.
    fn read_header(
        reader: &'r mut R,
        version: u8,
        keychain: &'k RefCell<Keychain>,
    ) -> BinResult<Self> {
        let record_type = u8::read(reader)?;
        // Up to v12, the length is stored on a single byte
        let length = utils::read_u16(reader, Endian::Little, (version <= 12,))?;
        let start = reader.stream_position()?;
        Ok(RecordContent {
            reader,
            version,
            keychain,
            record_type,
            length,
            start,
        })
    }

    /// Decodes and parses the content as `T`, then checks the record end byte.
    fn read<T>(&mut self, args: T::Args<'_>) -> BinResult<T>
    where
        T: BinRead,
    {
        let value = {
            let mut decoder = record_decoder(
                &mut *self.reader,
                self.record_type,
                self.version,
                self.keychain,
                self.length,
            );
            T::read_options(&mut decoder, Endian::Little, args)?
        };
        self.finish()?;
        Ok(value)
    }

    /// Reads the content as is, without decoding, then checks the record end byte.
    fn read_raw(&mut self) -> BinResult<Vec<u8>> {
        let value = Vec::<u8>::read_options(
            self.reader,
            Endian::Little,
            args! { count: self.length.into() },
        )?;
        self.finish()?;
        Ok(value)
    }

    /// Skips the content padding, if the parsed structure is shorter than the content, and
    /// checks the end byte.
    fn finish(&mut self) -> BinResult<()> {
        let end = self.start + u64::from(self.length);
        if self.reader.stream_position()? < end {
            self.reader.seek(SeekFrom::Start(end))?;
        }

        let pos = self.reader.stream_position()?;
        if u8::read(self.reader)? != END_BYTE {
            return Err(binrw::Error::AssertFail {
                pos,
                message: "invalid record end byte".to_owned(),
            });
        }
        Ok(())
    }
}
