use binrw::binread;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use serde::{Deserialize, Serialize};

/// Feature point of a record, which determines the AES key used to decrypt it.
///
/// Serialized with the names used by the DJI keychains API.
#[binread]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
#[br(repr(u16))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum FeaturePoint {
    #[serde(rename = "FR_Standardization_Feature_Base_1")]
    BaseFeature = 1,
    #[serde(rename = "FR_Standardization_Feature_Vision_2")]
    VisionFeature,
    #[serde(rename = "FR_Standardization_Feature_Waypoint_3")]
    WaypointFeature,
    #[serde(rename = "FR_Standardization_Feature_Agriculture_4")]
    AgricultureFeature,
    #[serde(rename = "FR_Standardization_Feature_AirLink_5")]
    AirLinkFeature,
    #[serde(rename = "FR_Standardization_Feature_AfterSales_6")]
    AfterSalesFeature,
    #[serde(rename = "FR_Standardization_Feature_DJIFlyCustom_7")]
    DJIFlyCustomFeature,
    #[serde(rename = "FR_Standardization_Feature_Plaintext_8")]
    PlaintextFeature,
    #[serde(rename = "FR_Standardization_Feature_FlightHub_9")]
    FlightHubFeature,
    #[serde(rename = "FR_Standardization_Feature_Gimbal_10")]
    GimbalFeature,
    #[serde(rename = "FR_Standardization_Feature_RC_11")]
    RCFeature,
    #[serde(rename = "FR_Standardization_Feature_Camera_12")]
    CameraFeature,
    #[serde(rename = "FR_Standardization_Feature_Battery_13")]
    BatteryFeature,
    #[serde(rename = "FR_Standardization_Feature_FlySafe_14")]
    FlySafeFeature,
    #[serde(rename = "FR_Standardization_Feature_Security_15")]
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

#[cfg(test)]
mod tests {
    use super::FeaturePoint::{self, *};

    const ALL: [FeaturePoint; 15] = [
        BaseFeature,
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
    ];

    #[test]
    fn serializes_with_api_names() {
        assert_eq!(
            serde_json::to_string(&BaseFeature).unwrap(),
            r#""FR_Standardization_Feature_Base_1""#
        );
        assert_eq!(
            serde_json::to_string(&SecurityFeature).unwrap(),
            r#""FR_Standardization_Feature_Security_15""#
        );
    }

    #[test]
    fn api_names_match_discriminants() {
        for feature_point in ALL {
            let name = serde_json::to_string(&feature_point).unwrap();
            assert!(
                name.ends_with(&format!("_{}\"", feature_point as u16)),
                "{name} doesn't match {feature_point:?} = {}",
                feature_point as u16
            );
            let parsed: FeaturePoint = serde_json::from_str(&name).unwrap();
            assert_eq!(parsed, feature_point);
        }
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(serde_json::from_str::<FeaturePoint>(r#""FR_Unknown""#).is_err());
    }
}
