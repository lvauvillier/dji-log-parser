use serde::Serialize;

use crate::layout::details::Details as LayoutDetails;
use crate::layout::details::Platform;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    /// Total flight time in seconds
    pub total_time: f32,
    /// Total distance flown in meters
    pub total_distance: f32,
    /// Maximum height reached during the flight in meters
    pub max_height: f32,
    /// Maximum horizontal speed reached during the flight in meters per second
    pub max_horizontal_speed: f32,
    /// Maximum vertical speed reached during the flight in meters per second
    pub max_vertical_speed: f32,
    /// Number of photos taken during the flight
    pub photo_num: i32,
    /// Total video recording time in seconds
    pub video_time: i64,
    /// Name of the aircraft
    pub aircraft_name: String,
    /// Serial number of the aircraft
    pub aircraft_sn: String,
    /// Serial number of the camera
    pub camera_sn: String,
    /// Serial number of the remote control
    pub rc_sn: String,
    /// The platform of the app used (e.g., iOS, Android)
    pub app_platform: Platform,
    /// Version of the app used
    pub app_version: String,
}

impl From<LayoutDetails> for Details {
    fn from(value: LayoutDetails) -> Self {
        Details {
            total_time: value.total_time as f32,
            total_distance: value.total_distance,
            max_height: value.max_height,
            max_horizontal_speed: value.max_horizontal_speed,
            max_vertical_speed: value.max_vertical_speed,
            photo_num: value.capture_num,
            video_time: value.video_time,
            aircraft_name: value.aircraft_name.clone(),
            aircraft_sn: value.aircraft_sn.clone(),
            camera_sn: value.camera_sn.clone(),
            rc_sn: value.rc_sn.clone(),
            app_platform: value.app_platform.clone(),
            app_version: value.app_version.clone(),
        }
    }
}
