use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct KeychainData {
    pub feature_point: FeaturePoint,
    pub inner: CommonData,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct TypeData {
    pub type_data: u8,
    pub inner: CommonData,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct CommonData {
    data_length: u16,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct InfoData {
    pub version_data: u8,
    pub info: CommonData,
    pub signature: CommonData,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct VersionData {
    pub version: u16,
    pub department: u8,
}

#[binread]
#[derive(Debug)]
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

impl FeaturePoint {
    // Method to convert a `FeaturePoint` enum variant to its string representation
    fn to_string(&self) -> &'static str {
        match self {
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
        }
    }
}
