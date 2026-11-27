use binrw::io::{Cursor, Read, Seek, SeekFrom};
use binrw::{args, BinRead, BinResult, Endian};
use serde::Serialize;
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::decoder::{record_decoder, xor_checksum_is_valid};
use crate::layout::details::ProductType;
use crate::utils;
use crate::Keychain;

pub mod adsb_flight_data;
pub mod aircraft_info;
pub mod app_gps;
pub mod app_joystick;
pub mod app_serious_warn;
pub mod app_tip;
pub mod app_warn;
pub mod camera;
pub mod center_battery;
pub mod component_serial;
pub mod custom;
pub mod deform;
pub mod firmware;
pub mod flight_controller_common_osd_field;
pub mod gimbal;
pub mod ground_station_push_data;
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
pub mod vision_group;

use adsb_flight_data::ADSBFlightData;
use aircraft_info::AircraftInfo;
use app_gps::AppGPS;
use app_joystick::AppJoystick;
use app_serious_warn::AppSeriousWarn;
use app_tip::AppTip;
use app_warn::AppWarn;
use camera::Camera;
use center_battery::CenterBattery;
use component_serial::ComponentSerial;
use custom::Custom;
use deform::Deform;
use firmware::Firmware;
use flight_controller_common_osd_field::FlightControllerCommonOSDField;
use gimbal::Gimbal;
use ground_station_push_data::GroundStationPushData;
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
use vision_group::VisionGroup;

const END_BYTE: u8 = 0xFF;
/// First bytes of the JPEG images written between the records: the image marker, or the image
const JPEG_STARTS: [[u8; 2]; 2] = [[0x39, 0x30], [0xFF, 0xD8]];

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
    VisionGroup(VisionGroup),
    AppJoystick(AppJoystick),
    ADSBFlightData(ADSBFlightData),
    AircraftInfo(AircraftInfo),
    GroundStationPushData(GroundStationPushData),
    FlightControllerCommonOSDField(FlightControllerCommonOSDField),
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
    ///
    /// A valid record whose data doesn't match the structure of its type is read as unknown,
    /// from the data already decoded: decoding it again would use the next iv of its feature.
    fn read_known<R: Read + Seek>(
        reader: &mut R,
        version: u8,
        product_type: ProductType,
        keychain: &RefCell<Keychain>,
    ) -> BinResult<Record> {
        let mut content = RecordContent::read_header(reader, version, keychain)?;
        Self::parse_known(&mut content, version, product_type).or_else(|error| {
            match content.unparsed.take() {
                Some(data) if content.length > 2 => Ok(Record::Unknown(content.record_type, data)),
                _ => Err(error),
            }
        })
    }

    fn parse_known<R: Read + Seek>(
        content: &mut RecordContent<'_, '_, R>,
        version: u8,
        product_type: ProductType,
    ) -> BinResult<Record> {
        let length = content.data_length();

        let record = match content.record_type {
            1 => Record::OSD(content.read::<OSD>(args! { version })?),
            2 => Record::Home(content.read::<Home>(args! { version })?),
            3 => Record::Gimbal(content.read::<Gimbal>(args! { version })?),
            4 => Record::RC(content.read::<RC>(args! { version, product_type })?),
            5 => Record::Custom(content.read::<Custom>(args! { version })?),
            6 => Record::Deform(content.read(())?),
            7 => Record::CenterBattery(content.read::<CenterBattery>(args! { version })?),
            8 => Record::SmartBattery(content.read(())?),
            9 => Record::AppTip(content.read::<AppTip>(args! { length })?),
            10 => Record::AppWarn(content.read::<AppWarn>(args! { length })?),
            11 => Record::RCGPS(content.read(())?),
            13 => Record::Recover(content.read::<Recover>(args! { length })?),
            14 => Record::AppGPS(content.read(())?),
            15 => Record::Firmware(content.read(())?),
            17 => Record::VisionGroup(content.read::<VisionGroup>((length,))?),
            19 => Record::MCParams(content.read(())?),
            22 => Record::SmartBatteryGroup(content.read::<SmartBatteryGroup>((version, length))?),
            24 => Record::AppSeriousWarn(content.read::<AppSeriousWarn>(args! { length })?),
            25 => Record::Camera(content.read(())?),
            26 => Record::ADSBFlightData(content.read::<ADSBFlightData>(args! { length })?),
            29 => Record::AppJoystick(content.read::<AppJoystick>(args! { length })?),
            33 => Record::VirtualStick(content.read(())?),
            40 => Record::ComponentSerial(content.read(())?),
            48 => Record::GroundStationPushData(
                content.read::<GroundStationPushData>(args! { length })?,
            ),
            49 => Record::OFDM(content.read(())?),
            // Not encoded
            50 => Record::KeyStorageRecover(content.read_raw()?),
            56 => Record::KeyStorage(content.read(())?),
            62 => Record::RCDisplayField(content.read(())?),
            63 => Record::FlightControllerCommonOSDField(
                content.read::<FlightControllerCommonOSDField>(args! { length })?,
            ),
            253 => Record::AircraftInfo(content.read(())?),
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
        if content.length <= 2 {
            return Err(binrw::Error::AssertFail {
                pos: content.start,
                message: "unknown record too short".to_owned(),
            });
        }

        let data = content.decode()?;
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

/// Zeros appended to the content of a record, read as the fields missing from older formats.
const SHORT_RECORD_PADDING: usize = 64;

/// A record being read: its header has been parsed, its content is read with [`Self::read`].
struct RecordContent<'r, 'k, R> {
    reader: &'r mut R,
    version: u8,
    keychain: &'k RefCell<Keychain>,
    record_type: u8,
    length: u16,
    /// Position of the content, right after the header
    start: u64,
    /// Records with an overflowed length are followed by an arbitrary byte instead of the end byte
    has_end_byte: bool,
    /// Decoded data of a valid record that doesn't match the structure of its type
    unparsed: Option<Vec<u8>>,
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
        let mut length = utils::read_u16(reader, Endian::Little, (version <= 12,))?;
        let start = reader.stream_position()?;

        let overflowed_length = if (7..=12).contains(&version) {
            Self::overflowed_length(reader, record_type, length, start)?
        } else {
            None
        };
        if let Some(overflowed_length) = overflowed_length {
            length = overflowed_length;
        }

        Ok(RecordContent {
            reader,
            version,
            keychain,
            record_type,
            length,
            start,
            has_end_byte: overflowed_length.is_none(),
            unparsed: None,
        })
    }

    /// Up to v12, the length of a record is kept on a single byte, which overflows from 256 bytes.
    /// The end byte is written at this length from the start of the content. For a record of 262
    /// bytes, the length is written as 6 and:
    ///
    /// - the end byte `0xFF` is written over the 7th byte of the content;
    /// - the byte after the content is never written, it is arbitrary.
    ///
    /// The real length is the written one plus 0, 256, 512 or 768. The checksum alone matches by
    /// chance once in 256 records: the first length whose checksum matches and that is followed
    /// by another record is kept, else the first one whose checksum matches.
    fn overflowed_length(
        reader: &mut R,
        record_type: u8,
        length: u16,
        start: u64,
    ) -> BinResult<Option<u16>> {
        let valid_lengths: Vec<u16> = [0, 256, 512, 768]
            .map(|overflow| length + overflow)
            .into_iter()
            .filter(|&length| Self::checksum_is_valid(reader, record_type, start, length))
            .collect();
        let real_length = valid_lengths
            .iter()
            .copied()
            .find(|&length| Self::record_starts_at(reader, start + u64::from(length) + 1))
            .or(valid_lengths.first().copied());

        reader.seek(SeekFrom::Start(start))?;
        Ok(real_length.filter(|&real_length| real_length != length))
    }

    /// Checks the checksum of the xor encoded content of `length` bytes at `start`.
    fn checksum_is_valid(reader: &mut R, record_type: u8, start: u64, length: u16) -> bool {
        let mut read = || -> BinResult<bool> {
            if length < 2 {
                return Ok(false);
            }
            reader.seek(SeekFrom::Start(start))?;
            let first = u8::read(reader)?;
            reader.seek(SeekFrom::Start(start + u64::from(length) - 1))?;
            let last = u8::read(reader)?;
            Ok(xor_checksum_is_valid(first, last, length, record_type))
        };
        read().unwrap_or(false)
    }

    /// Checks that a record, a JPEG image or the end of the data starts at `position`: a record
    /// has an end byte at its written length from the start of its content, and a valid checksum
    /// with one of its possible lengths.
    fn record_starts_at(reader: &mut R, position: u64) -> bool {
        let mut read = || -> BinResult<bool> {
            reader.seek(SeekFrom::Start(position))?;
            let record_type = match u8::read(reader) {
                Ok(record_type) => record_type,
                Err(error) if error.is_eof() => return Ok(true),
                Err(error) => return Err(error),
            };
            let length = u8::read(reader)?;
            if JPEG_STARTS.contains(&[record_type, length]) {
                return Ok(true);
            }
            let start = position + 2;
            reader.seek(SeekFrom::Start(start + u64::from(length)))?;
            if u8::read(reader)? != END_BYTE {
                return Ok(false);
            }
            Ok([0, 256, 512, 768].into_iter().any(|overflow| {
                Self::checksum_is_valid(reader, record_type, start, u16::from(length) + overflow)
            }))
        };
        read().unwrap_or(false)
    }

    /// Size of the decoded data. From v7, the content starts with the xor seed and ends with a
    /// checksum, which are not part of the data.
    fn data_length(&self) -> u16 {
        if self.version <= 6 {
            self.length
        } else {
            self.length.saturating_sub(2)
        }
    }

    /// Decodes the content and checks the record end byte, then parses the data as `T`.
    ///
    /// Records written by older apps can be shorter than the structure, which grew over time:
    /// the missing trailing fields are read as zeros. When the data doesn't match `T`, it is
    /// kept in `unparsed`.
    fn read<T>(&mut self, args: T::Args<'_>) -> BinResult<T>
    where
        T: BinRead,
    {
        let mut data = self.decode()?;
        let length = data.len();
        data.resize(length + SHORT_RECORD_PADDING, 0);

        T::read_options(&mut Cursor::new(&data), Endian::Little, args).inspect_err(|_| {
            data.truncate(length);
            self.unparsed = Some(data);
        })
    }

    /// Decodes the content, then checks the record end byte.
    fn decode(&mut self) -> BinResult<Vec<u8>> {
        let mut data = vec![0; self.data_length().into()];
        record_decoder(
            &mut *self.reader,
            self.record_type,
            self.version,
            self.keychain,
            self.length,
        )
        .read_exact(&mut data)?;
        self.finish()?;
        Ok(data)
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
        if u8::read(self.reader)? != END_BYTE && self.has_end_byte {
            return Err(binrw::Error::AssertFail {
                pos,
                message: "invalid record end byte".to_owned(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::xor_key;
    use crate::keychain::{FeaturePoint, KeychainFeaturePoint};
    use aes::cipher::block_padding::Pkcs7;
    use aes::cipher::{BlockModeEncrypt, KeyIvInit};
    use base64::engine::general_purpose::STANDARD as Base64Standard;
    use base64::Engine as _;

    /// Builds a record as written up to v6: type, length on one byte, content, end byte.
    fn raw_record(record_type: u8, content: &[u8]) -> Vec<u8> {
        let mut bytes = vec![record_type, content.len() as u8];
        bytes.extend_from_slice(content);
        bytes.push(END_BYTE);
        bytes
    }

    /// Xor encodes the content of a record: a seed, the data and a checksum.
    fn xor_content(record_type: u8, data: &[u8]) -> Vec<u8> {
        let seed = 0x42;
        let key = xor_key(seed, record_type);
        let mut decoded = data.to_vec();
        decoded.push(seed.wrapping_add(record_type));

        let mut content = vec![seed];
        content.extend(
            decoded
                .iter()
                .enumerate()
                .map(|(i, byte)| byte ^ key[i % 8]),
        );
        content
    }

    /// Builds a record as written from v7 up to v12: the content is xor encoded, starts with a
    /// seed and ends with a checksum. Only the low byte of the length is written.
    fn xor_record(record_type: u8, data: &[u8], end_byte: u8) -> Vec<u8> {
        let content = xor_content(record_type, data);
        let mut bytes = vec![record_type, content.len() as u8];
        bytes.extend(content);
        bytes.push(end_byte);
        bytes
    }

    /// Builds a record as written from v13: the data is AES encrypted, then xor encoded.
    fn aes_record(record_type: u8, data: &[u8], key: [u8; 32], iv: [u8; 16]) -> Vec<u8> {
        let mut buffer = data.to_vec();
        buffer.resize(data.len().next_multiple_of(16) + 16, 0);
        let encrypted = cbc::Encryptor::<aes::Aes256>::new(&key.into(), &iv.into())
            .encrypt_padded::<Pkcs7>(&mut buffer, data.len())
            .unwrap();

        let content = xor_content(record_type, encrypted);
        let mut bytes = vec![record_type];
        bytes.extend((content.len() as u16).to_le_bytes());
        bytes.extend(content);
        bytes.push(END_BYTE);
        bytes
    }

    fn read(version: u8, bytes: Vec<u8>) -> Record {
        let keychain = RefCell::new(Keychain::empty());
        Record::read(
            &mut Cursor::new(bytes),
            version,
            ProductType::None,
            &keychain,
        )
        .unwrap()
    }

    #[test]
    fn missing_fields_of_short_records_are_zeros() {
        // OSD records are 49 bytes long in some v3 logs, shorter than the structure
        let mut content = vec![0; 49];
        content[16..18].copy_from_slice(&1234i16.to_le_bytes());

        let Record::OSD(osd) = read(3, raw_record(1, &content)) else {
            panic!("not an OSD record");
        };
        assert_eq!(osd.altitude, 123.4);
    }

    #[test]
    fn next_record_is_read_after_invalid_data() {
        let mut bytes = vec![0xAA, 0xBB, END_BYTE];
        bytes.extend(raw_record(1, &[0; 53]));
        let keychain = RefCell::new(Keychain::empty());
        let mut reader = Cursor::new(bytes);

        let invalid = Record::read(&mut reader, 6, ProductType::None, &keychain).unwrap();
        assert!(matches!(invalid, Record::Invalid(data) if data == [0xAA, 0xBB, END_BYTE]));
        let next = Record::read(&mut reader, 6, ProductType::None, &keychain).unwrap();
        assert!(matches!(next, Record::OSD(_)));
    }

    #[test]
    fn embedded_image_is_read_up_to_its_end() {
        let image = [
            &[0xFF, 0xD8][..],
            // EXIF segment containing a thumbnail, with its own end of image marker
            &[0xFF, 0xE1, 0x00, 0x08, 0xFF, 0xD8, 0x01, 0x02, 0xFF, 0xD9],
            // Start of scan, then entropy-coded data with a stuffed 0xFF and a restart marker
            &[
                0xFF, 0xDA, 0x00, 0x02, 0x12, 0xFF, 0x00, 0x34, 0xFF, 0xD0, 0x56,
            ],
            &[0xFF, 0xD9],
        ]
        .concat();
        // Images are preceded by a 12345 marker
        let mut bytes = 12345i32.to_le_bytes().to_vec();
        bytes.extend(&image);
        bytes.extend(raw_record(1, &[0; 53]));
        let keychain = RefCell::new(Keychain::empty());
        let mut reader = Cursor::new(bytes);

        let jpeg = Record::read(&mut reader, 6, ProductType::None, &keychain).unwrap();
        assert!(matches!(jpeg, Record::JPEG(data) if data == image));
        let next = Record::read(&mut reader, 6, ProductType::None, &keychain).unwrap();
        assert!(matches!(next, Record::OSD(_)));
    }

    #[test]
    fn length_of_records_longer_than_255_bytes_is_found_with_the_checksum() {
        // Followed by an arbitrary byte instead of the end byte
        let data: Vec<u8> = (0..300).map(|i| i as u8).collect();
        let mut bytes = xor_record(16, &data, 0x00);
        bytes.extend(xor_record(1, &[0; 53], END_BYTE));
        let keychain = RefCell::new(Keychain::empty());
        let mut reader = Cursor::new(bytes);

        let long = Record::read(&mut reader, 8, ProductType::None, &keychain).unwrap();
        assert!(matches!(long, Record::Unknown(16, content) if content == data));
        let next = Record::read(&mut reader, 8, ProductType::None, &keychain).unwrap();
        assert!(matches!(next, Record::OSD(_)));
    }

    #[test]
    fn length_of_records_longer_than_255_bytes_is_found_when_the_shorter_one_matches_by_chance() {
        // Content of 302 bytes, written as 46: the checksum of 46 bytes matches, and the end
        // byte is written after them, over the data
        let mut data: Vec<u8> = (0..300).map(|i| i as u8).collect();
        data[44] = 0x42 + 16;
        let mut bytes = xor_record(16, &data, 0x00);
        bytes[2 + 46] = END_BYTE;
        bytes.extend(xor_record(1, &[0; 53], END_BYTE));
        let keychain = RefCell::new(Keychain::empty());
        let mut reader = Cursor::new(bytes);

        let long = Record::read(&mut reader, 8, ProductType::None, &keychain).unwrap();
        assert!(matches!(long, Record::Unknown(16, content) if content.len() == data.len()));
        let next = Record::read(&mut reader, 8, ProductType::None, &keychain).unwrap();
        assert!(matches!(next, Record::OSD(_)));
    }

    #[test]
    fn custom_timestamp_is_in_seconds_up_to_v2() {
        let mut content = vec![0; 18];
        content[10..].copy_from_slice(&1_418_287_736i64.to_le_bytes());

        let Record::Custom(custom) = read(2, raw_record(5, &content)) else {
            panic!("not a Custom record");
        };
        assert_eq!(
            custom.update_timestamp.to_rfc3339(),
            "2014-12-11T08:48:56+00:00"
        );
    }

    #[test]
    fn invalid_data_of_encrypted_records_is_kept_decoded() {
        let (key, iv) = ([7; 32], [9; 16]);
        let keychain = RefCell::new(
            Keychain::from_feature_points(&[KeychainFeaturePoint {
                feature_point: FeaturePoint::DJIFlyCustomFeature,
                aes_key: Base64Standard.encode(key),
                aes_iv: Base64Standard.encode(iv),
            }])
            .unwrap(),
        );
        // Custom data with a timestamp of 0, rejected
        let mut data = (1..=10).collect::<Vec<u8>>();
        data.resize(18, 0);

        let record = Record::read(
            &mut Cursor::new(aes_record(5, &data, key, iv)),
            14,
            ProductType::None,
            &keychain,
        )
        .unwrap();

        let Record::Unknown(5, unknown) = record else {
            panic!("not an unknown custom record: {record:?}");
        };
        assert_eq!(unknown[..18], data);
    }
}
