use serde::Serialize;

use crate::layout::details::Platform;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recover {
    /// The platform of the app used (e.g., iOS, Android)
    pub app_platform: Option<Platform>,
    /// Version of the app used
    pub app_version: String,
    /// Name of the aircraft
    pub aircraft_name: String,
    /// Serial number of the aircraft
    pub aircraft_sn: String,
    // Serial number of the camera
    pub camera_sn: String,
    /// Serial number of the remote control
    pub rc_sn: String,
    /// Serial number of the battery
    pub battery_sn: String,
}
