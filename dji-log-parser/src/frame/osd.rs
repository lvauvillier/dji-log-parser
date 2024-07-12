use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::osd::{
    AppCommand, BatteryType, DroneType, FlightAction, FlightMode, GoHomeStatus, ImuInitFailReason,
    MotorStartFailedCause, NonGPSCause,
};

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameOSD {
    /// Flight time in seconds
    pub fly_time: f32,
    /// Latitude in degrees
    pub latitude: f64,
    /// Longitude in degrees
    pub longitude: f64,
    /// Height above ground level in meters
    pub height: f32,
    /// Maximum height reached in meters
    pub height_max: f32,
    /// Visual Positioning System height in meters
    pub vps_height: f32,
    /// Altitude above sea level in meters
    pub altitude: f32,
    /// Speed along the X-axis in meters per second
    pub x_speed: f32,
    /// Maximum speed reached along the X-axis in meters per second
    pub x_speed_max: f32,
    /// Speed along the Y-axis in meters per second
    pub y_speed: f32,
    /// Maximum speed reached along the Y-axis in meters per second
    pub y_speed_max: f32,
    /// Vertical speed in meters per second
    pub z_speed: f32,
    /// Maximum vertical speed reached in meters per second
    pub z_speed_max: f32,
    /// Pitch angle in degrees
    pub pitch: f32,
    /// Roll angle in degrees
    pub roll: f32,
    /// Yaw angle in degrees
    pub yaw: f32,
    /// Current flight mode
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub flyc_state: Option<FlightMode>,
    /// Current app command
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub flyc_command: Option<AppCommand>,
    /// Current flight action
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub flight_action: Option<FlightAction>,
    /// Indicates if GPS is being used
    pub is_gpd_used: bool,
    /// Reason for not using GPS
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub non_gps_cause: Option<NonGPSCause>,
    /// Number of GPS satellites detected
    pub gps_num: u8,
    /// GPS signal level
    pub gps_level: u8,
    /// Type of drone
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub drone_type: Option<DroneType>,
    /// Indicates if obstacle avoidance is active
    pub is_swave_work: bool,
    /// Indicates if there's an error with obstacle avoidance
    pub wave_error: bool,
    /// Current status of the return-to-home function
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub go_home_status: Option<GoHomeStatus>,
    /// Type of battery
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub battery_type: Option<BatteryType>,
    /// Indicates if the drone is on the ground
    pub is_on_ground: bool,
    /// Indicates if the motor is running
    pub is_motor_on: bool,
    /// Indicates if the motor is blocked
    pub is_motor_blocked: bool,
    /// Reason for motor start failure
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub motor_start_failed_cause: Option<MotorStartFailedCause>,
    /// Indicates if the IMU is preheated
    pub is_imu_preheated: bool,
    /// Reason for IMU initialization failure
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub imu_init_fail_reason: Option<ImuInitFailReason>,
    /// Indicates if the accelerometer is over range
    pub is_acceletor_over_range: bool,
    /// Indicates if the barometer is malfunctioning in air
    pub is_barometer_dead_in_air: bool,
    /// Indicates if there's a compass error
    pub is_compass_error: bool,
    /// Indicates if the return-to-home height has been modified
    pub is_go_home_height_modified: bool,
    /// Indicates if Intelligent Orientation Control can work
    pub can_ioc_work: bool,
    /// Indicates if there's not enough force (e.g., low battery)
    pub is_not_enough_force: bool,
    /// Indicates if the drone is out of its flight limit
    pub is_out_of_limit: bool,
    /// Indicates if propeller catapult protection is active
    pub is_propeller_catapult: bool,
    /// Indicates if the drone is experiencing vibrations
    pub is_vibrating: bool,
    /// Indicates if vision positioning system is being used
    pub is_vision_used: bool,
    /// Battery voltage warning level
    pub voltage_warning: u8,
}
