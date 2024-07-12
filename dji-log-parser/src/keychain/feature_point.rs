use binrw::binread;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use serde::{Deserialize, Serialize, Serializer};

#[binread]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
#[br(repr(u16))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
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

impl FeaturePoint {
    pub fn from_record_type(record_type: u8, version: u8) -> Self {
        match record_type {
            // OSDFlightRecordDataType
            1 => FeaturePoint::BaseFeature,
            // OSDHomeFlightRecordDataType
            2 => FeaturePoint::BaseFeature,
            // GimbalFlightRecordDataType
            3 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::GimbalFeature
                }
            }
            // RCFlightRecordDataType
            4 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::RCFeature
                }
            }
            // CustomFlightRecordDataType
            5 => FeaturePoint::DJIFlyCustomFeature,
            // MCTripodStateFlightRecordDataType
            6 => FeaturePoint::BaseFeature,
            // CenterBatteryFlightRecordDataType
            7 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::BatteryFeature
                }
            }
            // PushedBatteryFlightRecordDataType
            8 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::BatteryFeature
                }
            }
            // ShowTipsFlightRecordDataType
            9 => FeaturePoint::DJIFlyCustomFeature,
            // ShowWarningFlightRecordDataType
            10 => FeaturePoint::DJIFlyCustomFeature,
            // RCPushGPSFlightRecordDataType
            11 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::RCFeature
                }
            }
            // RCDebugDataType
            12 => FeaturePoint::AfterSalesFeature,
            // RecoverInfoDataType
            13 => FeaturePoint::BaseFeature,
            // AppLocationDataType
            14 => FeaturePoint::BaseFeature,
            // FirmwareVersionType
            15 => FeaturePoint::BaseFeature,
            // OFDMDebugDataType
            16 => FeaturePoint::AfterSalesFeature,
            // VisionGroupDataType
            17 => FeaturePoint::VisionFeature,
            // VisionWaringStringDataType
            18 => FeaturePoint::VisionFeature,
            // MCParamDataType
            19 => FeaturePoint::AfterSalesFeature,
            // APPOperationDataType
            20 => FeaturePoint::DJIFlyCustomFeature,
            // AGOSDDataType
            21 => FeaturePoint::AgricultureFeature,
            // SmartBatteryGroupDataType
            22 => {
                if version == 13 {
                    FeaturePoint::AfterSalesFeature
                } else {
                    FeaturePoint::BatteryFeature
                }
            }
            // ShowSeriousWarningFlightRecordDataType
            24 => FeaturePoint::DJIFlyCustomFeature,
            // CameraInfoFlightRecordDataType
            25 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::CameraFeature
                }
            }
            // ADSBFlightDataDataType
            26 => FeaturePoint::AfterSalesFeature,
            // ADSBFlightOriginalDataType
            27 => FeaturePoint::AfterSalesFeature,
            // FlyForbidDBuuidDataType
            28 => {
                if version == 13 {
                    FeaturePoint::AfterSalesFeature
                } else {
                    FeaturePoint::FlySafeFeature
                }
            }
            // AppSpecialControlJoyStickDataType
            29 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::RCFeature
                }
            }
            // AppLowFreqCustomDataType
            30 => FeaturePoint::DJIFlyCustomFeature,
            // NavigationModeGroupedDataType
            31 => FeaturePoint::WaypointFeature,
            // GSMissionStatusDataType
            32 => FeaturePoint::WaypointFeature,
            // AppVirtualStickDataType
            33 => {
                if version == 13 {
                    FeaturePoint::BaseFeature
                } else {
                    FeaturePoint::RCFeature
                }
            }
            // GSMissionEventDataType
            34 => FeaturePoint::WaypointFeature,
            // WaypointMissionUploadDataType
            35 => FeaturePoint::WaypointFeature,
            // WaypointUploadDataType
            36 => FeaturePoint::WaypointFeature,
            // WaypointMissionDownloadDataType
            38 => FeaturePoint::WaypointFeature,
            // WaypointDownloadDataType
            39 => FeaturePoint::WaypointFeature,
            // ComponentSerialNumberDataType
            40 => FeaturePoint::BaseFeature,
            // AgricultureDisplayField
            41 => FeaturePoint::AgricultureFeature,
            // AgricultureRadarPush
            43 => FeaturePoint::AgricultureFeature,
            // AgricultureSpray
            44 => FeaturePoint::AgricultureFeature,
            // RTKDifferenceDataType
            45 => FeaturePoint::AgricultureFeature,
            // AgricultureFarmMissionInfo
            46 => FeaturePoint::AgricultureFeature,
            // AgricultureFarmTaskTeamDataType
            47 => FeaturePoint::AgricultureFeature,
            // AgricultureGroundStationPushData
            48 => FeaturePoint::AgricultureFeature,
            // AgricultureOFDMRadioSignalPush
            49 => FeaturePoint::AirLinkFeature,
            // FlightRecordRecover
            50 => FeaturePoint::PlaintextFeature,
            // FlySafeLimitInfoDataType
            51 => {
                if version == 13 {
                    FeaturePoint::AfterSalesFeature
                } else {
                    FeaturePoint::FlySafeFeature
                }
            }
            // FlySafeUnlockLicenseUserActionInfoDataType
            52 => {
                if version == 13 {
                    FeaturePoint::AfterSalesFeature
                } else {
                    FeaturePoint::FlySafeFeature
                }
            }
            // FlightHubInfoDataType
            53 => {
                if version == 13 {
                    FeaturePoint::AfterSalesFeature
                } else {
                    FeaturePoint::FlightHubFeature
                }
            }
            // GOBusinessDataType
            54 => FeaturePoint::DJIFlyCustomFeature,
            // Unknown
            55 => FeaturePoint::SecurityFeature,
            // KeyStorage
            56 => FeaturePoint::PlaintextFeature,
            // HealthGroupDataType
            58 => FeaturePoint::BaseFeature,
            // FCIMUInfoDataType
            59 => FeaturePoint::BaseFeature,
            // RemoteControllerDisplayField
            62 => FeaturePoint::RCFeature,
            // FlightControllerCommonOSDField
            63 => FeaturePoint::BaseFeature,
            // Default
            _ => FeaturePoint::PlaintextFeature,
        }
    }
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
