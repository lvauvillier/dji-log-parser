use chrono::{DateTime, Utc};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::layout::details::Details;
use crate::layout::details::Platform;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameDetails {
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
    /// Start time of the flight
    pub start_time: DateTime<Utc>,
}

impl FrameDetails {
    /// Creates FrameDetails from Details, but uses the most recent frame data 
    /// for serial numbers if available (from ComponentSerial records)
    pub fn from_details_and_frames(details: Details, frames: &[super::Frame]) -> Self {
        let mut frame_details = Self::from(details.clone());
        
        // Find the most recent frame with updated serial numbers from ComponentSerial records
        // We check if the frame serial is longer than the details serial, indicating it came from ComponentSerial
        if let Some(last_frame) = frames.last() {
            if last_frame.recover.aircraft_sn.len() > details.aircraft_sn.len() {
                frame_details.aircraft_sn = last_frame.recover.aircraft_sn.clone();
            }
            if last_frame.recover.camera_sn.len() > details.camera_sn.len() {
                frame_details.camera_sn = last_frame.recover.camera_sn.clone();
            }
            if last_frame.recover.rc_sn.len() > details.rc_sn.len() {
                frame_details.rc_sn = last_frame.recover.rc_sn.clone();
            }
        }
        
        frame_details
    }
}

impl From<Details> for FrameDetails {
    fn from(value: Details) -> Self {
        FrameDetails {
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
            start_time: value.start_time,
        }
    }
}
