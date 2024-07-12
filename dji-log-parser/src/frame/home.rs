use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::home::{CompassCalibrationState, GoHomeMode, IOCMode};

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameHome {
    /// Home point latitude in degrees
    pub latitude: f64,
    /// Home point longitude in degrees
    pub longitude: f64,
    /// Home point altitude in meters
    pub altitude: f32,
    /// Max allowed height in meters
    pub height_limit: f32,
    /// Indicates if home point is recorded
    pub is_home_record: bool,
    /// Current return-to-home mode
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_mode: Option<GoHomeMode>,
    /// Indicates if dynamic home point is enabled
    pub is_dynamic_home_point_enabled: bool,
    /// Indicates if the drone is near its distance limit
    pub is_near_distance_limit: bool,
    /// Indicates if the drone is near its height limit
    pub is_near_height_limit: bool,
    /// Indicates if compass calibration is in progress
    pub is_compass_calibrating: bool,
    /// Current state of compass calibration
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub compass_calibration_state: Option<CompassCalibrationState>,
    /// Indicates if multiple flight modes are enabled
    pub is_multiple_mode_enabled: bool,
    /// Indicates if beginner mode is active
    pub is_beginner_mode: bool,
    /// Indicates if Intelligent Orientation Control is enabled
    pub is_ioc_enabled: bool,
    /// Current Intelligent Orientation Control mode
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub ioc_mode: Option<IOCMode>,
    /// Return-to-home height in meters
    pub go_home_height: u16,
    /// Intelligent Orientation Control course lock angle
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub ioc_course_lock_angle: Option<i16>,
    /// Maximum allowed height for the drone in meters
    pub max_allowed_height: f32,
    /// Index of the current flight record
    pub current_flight_record_index: u16,
}
