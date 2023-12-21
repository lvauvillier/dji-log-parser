use binrw::{binread, io::NoSeek, parser, BinResult};
use serde::{Deserialize, Serialize, Serializer};

use crate::decoder::Decoder;

#[binread]
#[derive(Debug)]
#[br(little, import(version: u8))]
pub enum Record {
    #[br(magic = 56u8)]
    KeyStorage(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(pad_size_to = self_0, map_stream = |reader| NoSeek::new(Decoder::new(reader, 56)))]
        KeyStorage,
        #[br(temp)] u8, // end byte
    ),
    #[br(magic = 50u8)]
    FlightRecordRecover(
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(count = self_0)] Vec<u8>,
        #[br(temp)] u8, // end byte
    ),
    Unknown(
        u8, // record_type
        #[br(temp, args(version <= 12), parse_with = read_u16)] u16,
        #[br(count = self_1)] Vec<u8>,
        #[br(temp)] u8, // end byte
    ),
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
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[br(repr(u16))]
pub enum FeaturePoint {
    BaseFeature = 1,
    VisionFeature,
    WaypointFeature,
    AgricultureFeature,
    AirLinkFeature,
    AfterSalesFeature,
    DJIFlyCustomFeature,
    PlaintextFeature,
    FlightHubFeature,
    GimbalFeature,
    RCFeature,
    CameraFeature,
    BatteryFeature,
    FlySafeFeature,
    SecurityFeature,
}

impl Serialize for FeaturePoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let feature_point_string = match self {
            FeaturePoint::BaseFeature => "FR_Standardization_Feature_Base_1",
            FeaturePoint::VisionFeature => "FR_Standardization_Feature_Vision_2",
            FeaturePoint::WaypointFeature => "FR_Standardization_Feature_Waypoint_3",
            FeaturePoint::AgricultureFeature => "FR_Standardization_Feature_Agriculture_4",
            FeaturePoint::AirLinkFeature => "FR_Standardization_Feature_AirLink_5",
            FeaturePoint::AfterSalesFeature => "FR_Standardization_Feature_AfterSales_6",
            FeaturePoint::DJIFlyCustomFeature => "FR_Standardization_Feature_DJIFlyCustom_7",
            FeaturePoint::PlaintextFeature => "FR_Standardization_Feature_Plaintext_8",
            FeaturePoint::FlightHubFeature => "FR_Standardization_Feature_FlightHub_9",
            FeaturePoint::GimbalFeature => "FR_Standardization_Feature_Gimbal_10",
            FeaturePoint::RCFeature => "FR_Standardization_Feature_RC_11",
            FeaturePoint::CameraFeature => "FR_Standardization_Feature_Camera_12",
            FeaturePoint::BatteryFeature => "FR_Standardization_Feature_Battery_13",
            FeaturePoint::FlySafeFeature => "FR_Standardization_Feature_FlySafe_14",
            FeaturePoint::SecurityFeature => "FR_Standardization_Feature_Security_15",
        };
        serializer.serialize_str(feature_point_string)
    }
}

impl<'de> Deserialize<'de> for FeaturePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "FR_Standardization_Feature_Base_1" => Ok(FeaturePoint::BaseFeature),
            "FR_Standardization_Feature_Vision_2" => Ok(FeaturePoint::VisionFeature),
            "FR_Standardization_Feature_Waypoint_3" => Ok(FeaturePoint::WaypointFeature),
            "FR_Standardization_Feature_Agriculture_4" => Ok(FeaturePoint::AgricultureFeature),
            "FR_Standardization_Feature_AirLink_5" => Ok(FeaturePoint::AirLinkFeature),
            "FR_Standardization_Feature_AfterSales_6" => Ok(FeaturePoint::AfterSalesFeature),
            "FR_Standardization_Feature_DJIFlyCustom_7" => Ok(FeaturePoint::DJIFlyCustomFeature),
            "FR_Standardization_Feature_Plaintext_8" => Ok(FeaturePoint::PlaintextFeature),
            "FR_Standardization_Feature_FlightHub_9" => Ok(FeaturePoint::FlightHubFeature),
            "FR_Standardization_Feature_Gimbal_10" => Ok(FeaturePoint::GimbalFeature),
            "FR_Standardization_Feature_RC_11" => Ok(FeaturePoint::RCFeature),
            "FR_Standardization_Feature_Camera_12" => Ok(FeaturePoint::CameraFeature),
            "FR_Standardization_Feature_Battery_13" => Ok(FeaturePoint::BatteryFeature),
            "FR_Standardization_Feature_FlySafe_14" => Ok(FeaturePoint::FlySafeFeature),
            "FR_Standardization_Feature_Security_15" => Ok(FeaturePoint::SecurityFeature),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid feature point: {}",
                s
            ))),
        }
    }
}
