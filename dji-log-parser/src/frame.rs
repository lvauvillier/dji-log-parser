use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::layout::details::{Details, Platform};
use crate::record::camera::SDCardState;
use crate::record::gimbal::GimbalMode;
use crate::record::home::{CompassCalibrationState, GoHomeMode, IOCMode};
use crate::record::osd::{
    AppCommand, BatteryType, DroneType, FlightAction, FlightMode, GoHomeStatus, GroundOrSky,
    ImuInitFailReason, MotorStartFailedCause, NonGPSCause,
};
use crate::record::smart_battery_group::SmartBatteryGroup;
use crate::record::Record;
use crate::utils::append_message;

/// Represents a normalized frame of data from a DJI log.
///
/// A `Frame` is a standardized representation of log data, normalized across
/// different log versions. It provides a consistent and easy-to-use format
/// for analyzing and processing DJI log information.
///
#[derive(Serialize, Debug, Default, Clone)]
pub struct Frame {
    /// Custom date and time of the frame
    #[serde(rename = "CUSTOM.dateTime")]
    pub custom_date_time: DateTime<Utc>,

    /// Flight time in seconds
    #[serde(rename = "OSD.flyTime")]
    pub osd_fly_time: f32,
    /// Latitude in degrees
    #[serde(rename = "OSD.lalitude")]
    pub osd_latitude: f64,
    /// Longitude in degrees
    #[serde(rename = "OSD.longitude")]
    pub osd_longitude: f64,
    /// Height above ground level in meters
    #[serde(rename = "OSD.height")]
    pub osd_height: f32,
    /// Maximum height reached in meters
    #[serde(rename = "OSD.heightMax")]
    pub osd_height_max: f32,
    /// Visual Positioning System height in meters
    #[serde(rename = "OSD.vpsHeight")]
    pub osd_vps_height: f32,
    /// Altitude above sea level in meters
    #[serde(rename = "OSD.altitude")]
    pub osd_altitude: f32,
    /// Speed along the X-axis in meters per second
    #[serde(rename = "OSD.xSpeed")]
    pub osd_x_speed: f32,
    /// Maximum speed reached along the X-axis in meters per second
    #[serde(rename = "OSD.xSpeedMax")]
    pub osd_x_speed_max: f32,
    /// Speed along the Y-axis in meters per second
    #[serde(rename = "OSD.ySpeed")]
    pub osd_y_speed: f32,
    /// Maximum speed reached along the Y-axis in meters per second
    #[serde(rename = "OSD.ySpeedMax")]
    pub osd_y_speed_max: f32,
    /// Vertical speed in meters per second
    #[serde(rename = "OSD.zSpeed")]
    pub osd_z_speed: f32,
    /// Maximum vertical speed reached in meters per second
    #[serde(rename = "OSD.zSpeedMax")]
    pub osd_z_speed_max: f32,
    /// Pitch angle in degrees
    #[serde(rename = "OSD.pitch")]
    pub osd_pitch: f32,
    /// Roll angle in degrees
    #[serde(rename = "OSD.roll")]
    pub osd_roll: f32,
    /// Yaw angle in degrees
    #[serde(rename = "OSD.yaw")]
    pub osd_yaw: f32,
    /// Current flight mode
    #[serde(rename = "OSD.flycState")]
    pub osd_flyc_state: Option<FlightMode>,
    /// Current app command
    #[serde(rename = "OSD.flycCommand")]
    pub osd_flyc_command: Option<AppCommand>,
    /// Current flight action
    #[serde(rename = "OSD.flightAction")]
    pub osd_flight_action: Option<FlightAction>,
    /// Indicates if GPS is being used
    #[serde(rename = "OSD.isGPSUsed")]
    pub osd_is_gpd_used: bool,
    /// Reason for not using GPS
    #[serde(rename = "OSD.nonGPSCause")]
    pub osd_non_gps_cause: Option<NonGPSCause>,
    /// Number of GPS satellites detected
    #[serde(rename = "OSD.gpsNum")]
    pub osd_gps_num: u8,
    /// GPS signal level
    #[serde(rename = "OSD.gpsLevel")]
    pub osd_gps_level: u8,
    /// Type of drone
    #[serde(rename = "OSD.droneType")]
    pub osd_drone_type: Option<DroneType>,
    /// Indicates if obstacle avoidance is active
    #[serde(rename = "OSD.isSwaveWork")]
    pub osd_is_swave_work: bool,
    /// Indicates if there's an error with obstacle avoidance
    #[serde(rename = "OSD.waveError")]
    pub osd_wave_error: bool,
    /// Current status of the return-to-home function
    #[serde(rename = "OSD.goHomeStatus")]
    pub osd_go_home_status: Option<GoHomeStatus>,
    /// Type of battery
    #[serde(rename = "OSD.batteryType")]
    pub osd_battery_type: Option<BatteryType>,
    /// Indicates if the drone is on the ground
    #[serde(rename = "OSD.isOnGround")]
    pub osd_is_on_ground: bool,
    /// Indicates if the motor is running
    #[serde(rename = "OSD.isMotorOn")]
    pub osd_is_motor_on: bool,
    /// Indicates if the motor is blocked
    #[serde(rename = "OSD.isMotorBlocked")]
    pub osd_is_motor_blocked: bool,
    /// Reason for motor start failure
    #[serde(rename = "OSD.motorStartFailedCause")]
    pub osd_motor_start_failed_cause: Option<MotorStartFailedCause>,
    /// Indicates if the IMU is preheated
    #[serde(rename = "OSD.isImuPreheated")]
    pub osd_is_imu_preheated: bool,
    /// Reason for IMU initialization failure
    #[serde(rename = "OSD.imuInitFailReason")]
    pub osd_imu_init_fail_reason: Option<ImuInitFailReason>,
    /// Indicates if the accelerometer is over range
    #[serde(rename = "OSD.isAcceleratorOverRange")]
    pub osd_is_acceletor_over_range: bool,
    /// Indicates if the barometer is malfunctioning in air
    #[serde(rename = "OSD.isBarometerDeadInAir")]
    pub osd_is_barometer_dead_in_air: bool,
    /// Indicates if there's a compass error
    #[serde(rename = "OSD.isCompassError")]
    pub osd_is_compass_error: bool,
    /// Indicates if the return-to-home height has been modified
    #[serde(rename = "OSD.isGoHomeHeightModified")]
    pub osd_is_go_home_height_modified: bool,
    /// Indicates if Intelligent Orientation Control can work
    #[serde(rename = "OSD.canIOCWork")]
    pub osd_can_ioc_work: bool,
    /// Indicates if there's not enough force (e.g., low battery)
    #[serde(rename = "OSD.isNotEnoughForce")]
    pub osd_is_not_enough_force: bool,
    /// Indicates if the drone is out of its flight limit
    #[serde(rename = "OSD.isOutOfLimit")]
    pub osd_is_out_of_limit: bool,
    /// Indicates if propeller catapult protection is active
    #[serde(rename = "OSD.isPropellerCatapult")]
    pub osd_is_propeller_catapult: bool,
    /// Indicates if the drone is experiencing vibrations
    #[serde(rename = "OSD.isVibrating")]
    pub osd_is_vibrating: bool,
    /// Indicates if vision positioning system is being used
    #[serde(rename = "OSD.isVisionUsed")]
    pub osd_is_vision_used: bool,
    /// Battery voltage warning level
    #[serde(rename = "OSD.voltageWarning")]
    pub osd_voltage_warning: u8,

    /// Current gimbal mode
    #[serde(rename = "GIMBAL.mode")]
    pub gimbal_mode: Option<GimbalMode>,
    /// Gimbal pitch angle in degrees
    #[serde(rename = "GIMBAL.pitch")]
    pub gimbal_pitch: f32,
    /// Gimbal roll angle in degrees
    #[serde(rename = "GIMBAL.roll")]
    pub gimbal_roll: f32,
    /// Gimbal yaw angle in degrees
    #[serde(rename = "GIMBAL.yaw")]
    pub gimbal_yaw: f32,
    /// Indicates if gimbal pitch is at its limit
    #[serde(rename = "GIMBAL.isPitchAtLimit")]
    pub gimbal_is_pitch_at_limit: bool,
    /// Indicates if gimbal roll is at its limit
    #[serde(rename = "GIMBAL.isRollAtLimit")]
    pub gimbal_is_roll_at_limit: bool,
    /// Indicates if gimbal yaw is at its limit
    #[serde(rename = "GIMBAL.isYawAtLimit")]
    pub gimbal_is_yaw_at_limit: bool,
    /// Indicates if the gimbal is stuck
    #[serde(rename = "GIMBAL.isStuck")]
    pub gimbal_is_stuck: bool,

    /// Indicates if the camera is in photo mode
    #[serde(rename = "CAMERA.isPhoto")]
    pub camera_is_photo: bool,
    /// Indicates if the camera is in video mode
    #[serde(rename = "CAMERA.isVideo")]
    pub camera_is_video: bool,
    /// Indicates if an SD card is inserted
    #[serde(rename = "CAMERA.sdCardIsInserted")]
    pub camera_sd_card_is_inserted: bool,
    /// Current state of the SD card
    #[serde(rename = "CAMERA.sdCardState")]
    pub camera_sd_card_state: Option<SDCardState>,

    /// Downlink signal strength
    #[serde(rename = "RC.downlinkSignal")]
    pub rc_downlink_signal: Option<u8>,
    /// Uplink signal strength
    #[serde(rename = "RC.uplinkSignal")]
    pub rc_uplink_signal: Option<u8>,
    /// Right stick horizontal position (aileron)
    #[serde(rename = "RC.aileron")]
    pub rc_aileron: u16,
    /// Right stick vertical position (elevator)
    #[serde(rename = "RC.elevator")]
    pub rc_elevator: u16,
    /// Left stick vertical position (throttle)
    #[serde(rename = "RC.throttle")]
    pub rc_throttle: u16,
    /// Left stick horizontal position (rudder)
    #[serde(rename = "RC.rudder")]
    pub rc_rudder: u16,

    /// Battery charge level in percentage
    #[serde(rename = "BATTERY.chargeLevel")]
    pub battery_charge_level: u8,
    /// Battery voltage
    #[serde(rename = "BATTERY.voltage")]
    pub battery_voltage: f32,
    /// Battery current
    #[serde(rename = "BATTERY.current")]
    pub battery_current: f32,
    /// Current battery capacity
    #[serde(rename = "BATTERY.currentCapacity")]
    pub battery_current_capacity: u32,
    /// Full battery capacity
    #[serde(rename = "BATTERY.fullCapacity")]
    pub battery_full_capacity: u32,
    /// Voltage of battery cell 1
    #[serde(rename = "BATTERY.cellVoltage1")]
    pub battery_cell_voltage1: f32,
    /// Voltage of battery cell 2
    #[serde(rename = "BATTERY.cellVoltage2")]
    pub battery_cell_voltage2: f32,
    /// Voltage of battery cell 2
    #[serde(rename = "BATTERY.cellVoltage3")]
    pub battery_cell_voltage3: f32,
    /// Voltage of battery cell 4
    #[serde(rename = "BATTERY.cellVoltage4")]
    pub battery_cell_voltage4: f32,
    /// Deviation in cell voltages
    #[serde(rename = "BATTERY.cellVoltageDeviation")]
    pub battery_cell_voltage_deviation: f32,
    /// Maximum deviation in cell voltages
    #[serde(rename = "BATTERY.maxCellVoltageDeviation")]
    pub battery_max_cell_voltage_deviation: f32,
    /// Battery temperature
    #[serde(rename = "BATTERY.temperature")]
    pub battery_temperature: f32,
    /// Minimum battery temperature
    #[serde(rename = "BATTERY.minTemperature")]
    pub battery_min_temperature: f32,
    /// Maximum battery temperature
    #[serde(rename = "BATTERY.maxTemperature")]
    pub battery_max_temperature: f32,

    /// Home point latitude in degrees
    #[serde(rename = "HOME.latitude")]
    pub home_latitude: f64,
    /// Home point longitude in degrees
    #[serde(rename = "HOME.longitude")]
    pub home_longitude: f64,
    /// Home point altitude in meters
    #[serde(rename = "HOME.altitude")]
    pub home_altitude: f32,
    /// Max allowed height in meters
    #[serde(rename = "HOME.heightLimit")]
    pub home_height_limit: f32,
    /// Indicates if home point is recorded
    #[serde(rename = "HOME.isHomeRecord")]
    pub home_is_home_record: bool,
    /// Current return-to-home mode
    #[serde(rename = "HOME.goHomeMode")]
    pub home_go_home_mode: Option<GoHomeMode>,
    /// Indicates if dynamic home point is enabled
    #[serde(rename = "HOME.isDynamicHomePointEnabled")]
    pub home_is_dynamic_home_point_enabled: bool,
    /// Indicates if the drone is near its distance limit
    #[serde(rename = "HOME.isNearDistanceLimit")]
    pub home_is_near_distance_limit: bool,
    /// Indicates if the drone is near its height limit
    #[serde(rename = "HOME.isNearHeightLimit")]
    pub home_is_near_height_limit: bool,
    /// Indicates if compass calibration is in progress
    #[serde(rename = "HOME.isCompassCalibrating")]
    pub home_is_compass_calibrating: bool,
    /// Current state of compass calibration
    #[serde(rename = "HOME.compassCalibrationState")]
    pub home_compass_calibration_state: Option<CompassCalibrationState>,
    /// Indicates if multiple flight modes are enabled
    #[serde(rename = "HOME.isMultipleModeEnabled")]
    pub home_is_multiple_mode_enabled: bool,
    /// Indicates if beginner mode is active
    #[serde(rename = "HOME.isBeginnerMode")]
    pub home_is_beginner_mode: bool,
    /// Indicates if Intelligent Orientation Control is enabled
    #[serde(rename = "HOME.isIOCEnabled")]
    pub home_is_ioc_enabled: bool,
    /// Current Intelligent Orientation Control mode
    #[serde(rename = "HOME.IOCMode")]
    pub home_ioc_mode: Option<IOCMode>,
    /// Return-to-home height in meters
    #[serde(rename = "HOME.goHomeHeight")]
    pub home_go_home_height: u16,
    /// Intelligent Orientation Control course lock angle
    #[serde(rename = "HOME.IOCCourseLockAngle")]
    pub home_ioc_course_lock_angle: Option<i16>,
    /// Maximum allowed height for the drone in meters
    #[serde(rename = "HOME.maxAllowedHeight")]
    pub home_max_allowed_height: f32,
    /// Index of the current flight record
    #[serde(rename = "HOME.currentFlightRecordIndex")]
    pub home_current_flight_record_index: u16,

    /// The platform of the app used (e.g., iOS, Android)
    #[serde(rename = "RECOVER.appPlatform")]
    pub recover_app_platform: Option<Platform>,
    /// Version of the app used
    #[serde(rename = "RECOVER.appVersion")]
    pub recover_app_version: String,
    /// Name of the aircraft
    #[serde(rename = "RECOVER.aircraftName")]
    pub recover_aircraft_name: String,
    /// Serial number of the aircraft
    #[serde(rename = "RECOVER.aircraftSerial")]
    pub recover_aircraft_sn: String,
    // Serial number of the camera
    #[serde(rename = "RECOVER.cameraSerial")]
    pub recover_camera_sn: String,
    /// Serial number of the remote control
    #[serde(rename = "RECOVER.rcSerial")]
    pub recover_rc_sn: String,
    /// Serial number of the battery
    #[serde(rename = "RECOVER.batterySerial")]
    pub recover_battery_sn: String,

    #[serde(rename = "APP.tip")]
    pub app_tip: String,
    #[serde(rename = "APP.warn")]
    pub app_warn: String,

    /// Total flight time in seconds
    #[serde(rename = "DETAILS.totalTime")]
    pub details_total_time: f32,
    /// Total distance flown in meters
    #[serde(rename = "DETAILS.totalDistance")]
    pub details_total_distance: f32,
    /// Maximum height reached during the flight in meters
    #[serde(rename = "DETAILS.maxHeight")]
    pub details_max_height: f32,
    /// Maximum horizontal speed reached during the flight in meters per second
    #[serde(rename = "DETAILS.maxHorizontalSpeed")]
    pub details_max_horizontal_speed: f32,
    /// Maximum vertical speed reached during the flight in meters per second
    #[serde(rename = "DETAILS.maxVerticalSpeed")]
    pub details_max_vertical_speed: f32,
    /// Number of photos taken during the flight
    #[serde(rename = "DETAILS.photoNum")]
    pub details_photo_num: i32,
    /// Total video recording time in seconds
    #[serde(rename = "DETAILS.videoTime")]
    pub details_video_time: i64,
    /// Name of the aircraft
    #[serde(rename = "DETAILS.aircraftName")]
    pub details_aircraft_name: String,
    /// Serial number of the aircraft
    #[serde(rename = "DETAILS.aircraftSerial")]
    pub details_aircraft_sn: String,
    /// Serial number of the camera
    #[serde(rename = "DETAILS.cameraSerial")]
    pub details_camera_sn: String,
    /// Serial number of the remote control
    #[serde(rename = "DETAILS.rcSerial")]
    pub details_rc_sn: String,
    /// The platform of the app used (e.g., iOS, Android)
    #[serde(rename = "DETAILS.appPlatform")]
    pub details_app_platform: Option<Platform>,
    /// Version of the app used
    #[serde(rename = "DETAILS.appVersion")]
    pub details_app_version: String,
}

/// Converts a vector of `Record` objects into a vector of `Frame` objects.
///
/// This function takes a list of `Record` objects and transforms each one into a
/// corresponding `Frame` object. The transformation process normalizes the data
/// across different log versions, creating a standardized format that's easier
/// to work with.
///
/// # Arguments
/// - `records`: A vector of `Record` objects representing the raw log data.
///
/// # Returns
/// - `Vec<Frame>`: A vector of `Frame` objects representing the normalized log data.
///   Each `Frame` corresponds to one or more `Record` objects, depending on the
///   specific normalization logic.
///
pub fn records_to_frames(records: Vec<Record>, details: Details) -> Vec<Frame> {
    let mut frames = Vec::new();

    //let mut frames = vec![];
    let mut frame = Frame::default();
    let mut frame_index = 0;

    for record in records {
        match record {
            Record::OSD(osd) => {
                // Push a new frame
                if frame_index > 0 {
                    frames.push(frame.clone());
                }

                // Reset non persistant values (one time events)
                frame.camera_is_photo = bool::default();
                frame.camera_is_video = bool::default();
                frame.app_tip = String::default();
                frame.app_warn = String::default();

                // Add details values
                frame.details_total_time = details.total_time as f32;
                frame.details_total_distance = details.total_distance;
                frame.details_max_height = details.max_height;
                frame.details_max_horizontal_speed = details.max_horizontal_speed;
                frame.details_max_vertical_speed = details.max_vertical_speed;
                frame.details_photo_num = details.capture_num;
                frame.details_video_time = details.video_time;
                frame.details_aircraft_name = details.aircraft_name.clone();
                frame.details_aircraft_sn = details.aircraft_sn.clone();
                frame.details_camera_sn = details.camera_sn.clone();
                frame.details_rc_sn = details.rc_sn.clone();
                frame.details_app_platform = Some(details.app_platform.clone());
                frame.details_app_version = details.app_version.clone();

                // Fill OSD record
                frame.osd_fly_time = osd.fly_time;
                frame.osd_latitude = osd.latitude;
                frame.osd_longitude = osd.longitude;
                // Fix altitude by adding the home point altitude
                frame.osd_altitude = osd.altitude + frame.home_altitude;
                frame.osd_height = osd.altitude;
                frame.osd_vps_height = osd.s_wave_height;
                if frame.osd_height_max < osd.altitude {
                    frame.osd_height_max = osd.altitude;
                }
                frame.osd_x_speed = osd.speed_x;
                if frame.osd_x_speed_max < osd.speed_x {
                    frame.osd_x_speed_max = osd.speed_x;
                }
                frame.osd_y_speed = osd.speed_y;
                if frame.osd_y_speed_max < osd.speed_y {
                    frame.osd_y_speed_max = osd.speed_y;
                }
                frame.osd_z_speed = osd.speed_z;
                if frame.osd_z_speed_max < osd.speed_z {
                    frame.osd_z_speed_max = osd.speed_z;
                }
                frame.osd_pitch = osd.pitch;
                frame.osd_yaw = osd.yaw;
                frame.osd_roll = osd.roll;

                if frame.osd_flyc_state != Some(osd.flight_mode) {
                    frame.app_tip = append_message(
                        frame.app_tip,
                        format!("Flight mode changed to {:?}.", osd.flight_mode),
                    );
                }
                frame.osd_flyc_state = Some(osd.flight_mode);
                if let AppCommand::Unknown(0) = osd.app_command {
                    frame.osd_flyc_command = None;
                } else {
                    frame.osd_flyc_command = Some(osd.app_command);
                }
                frame.osd_flight_action = Some(osd.flight_action);
                frame.osd_gps_num = osd.gps_num;
                frame.osd_gps_level = osd.gps_level;
                frame.osd_is_gpd_used = osd.is_gps_valid;
                frame.osd_non_gps_cause = Some(osd.non_gps_cause);
                frame.osd_drone_type = Some(osd.drone_type);
                frame.osd_is_swave_work = osd.is_swave_work;
                frame.osd_wave_error = osd.wave_error;
                frame.osd_go_home_status = Some(osd.go_home_status);
                frame.osd_battery_type = Some(osd.battery_type);
                frame.osd_is_on_ground = osd.ground_or_sky == GroundOrSky::Ground;
                frame.osd_is_motor_on = osd.is_motor_up;
                frame.osd_is_motor_blocked = osd.is_motor_blocked;
                frame.osd_motor_start_failed_cause = Some(osd.motor_start_failed_cause);
                frame.osd_is_imu_preheated = osd.is_imu_preheated;
                frame.osd_imu_init_fail_reason = Some(osd.imu_init_fail_reason);
                frame.osd_is_acceletor_over_range = osd.is_acceletor_over_range;
                frame.osd_is_barometer_dead_in_air = osd.is_barometer_dead_in_air;
                frame.osd_is_compass_error = osd.is_compass_error;
                frame.osd_is_go_home_height_modified = osd.is_go_home_height_modified;
                frame.osd_can_ioc_work = osd.can_ioc_work;
                frame.osd_is_not_enough_force = osd.is_not_enough_force;
                frame.osd_is_out_of_limit = osd.is_out_of_limit;
                frame.osd_is_propeller_catapult = osd.is_propeller_catapult;
                frame.osd_is_vibrating = osd.is_vibrating;
                frame.osd_is_vision_used = osd.is_vision_used;
                frame.osd_voltage_warning = osd.voltage_warning;

                frame_index = frame_index + 1;
            }
            Record::Gimbal(gimbal) => {
                frame.gimbal_mode = Some(gimbal.mode);
                frame.gimbal_pitch = gimbal.pitch;
                frame.gimbal_roll = gimbal.roll;
                frame.gimbal_yaw = gimbal.yaw;
                if !frame.gimbal_is_pitch_at_limit && gimbal.is_pitch_at_limit {
                    frame.app_tip =
                        append_message(frame.app_tip, "Gimbal pitch axis endpoint reached.")
                }
                frame.gimbal_is_pitch_at_limit = gimbal.is_pitch_at_limit;
                if !frame.gimbal_is_roll_at_limit && gimbal.is_roll_at_limit {
                    frame.app_tip =
                        append_message(frame.app_tip, "Gimbal roll axis endpoint reached.")
                }
                frame.gimbal_is_roll_at_limit = gimbal.is_roll_at_limit;
                if !frame.gimbal_is_yaw_at_limit && gimbal.is_yaw_at_limit {
                    frame.app_tip =
                        append_message(frame.app_tip, "Gimbal yaw axis endpoint reached.")
                }
                frame.gimbal_is_yaw_at_limit = gimbal.is_yaw_at_limit;
                frame.gimbal_is_stuck = gimbal.is_stuck;
            }
            Record::Camera(camera) => {
                frame.camera_is_photo = camera.is_shooting_single_photo;
                frame.camera_is_video = camera.is_recording;
                frame.camera_sd_card_is_inserted = camera.has_sd_card;
                frame.camera_sd_card_state = Some(camera.sd_card_state);
            }
            Record::RC(rc) => {
                frame.rc_aileron = rc.aileron;
                frame.rc_elevator = rc.elevator;
                frame.rc_throttle = rc.throttle;
                frame.rc_rudder = rc.rudder;
            }
            Record::RCDisplayField(rc) => {
                frame.rc_aileron = rc.aileron;
                frame.rc_elevator = rc.elevator;
                frame.rc_throttle = rc.throttle;
                frame.rc_rudder = rc.rudder;
            }
            Record::CenterBattery(battery) => {
                frame.battery_charge_level = battery.relative_capacity;
                frame.battery_voltage = battery.voltage;
                frame.battery_current_capacity = battery.current_capacity as u32;
                frame.battery_full_capacity = battery.full_capacity as u32;
                frame.battery_full_capacity = battery.full_capacity as u32;

                frame.battery_cell_voltage1 = battery.voltage_cell1;
                frame.battery_cell_voltage2 = battery.voltage_cell2;
                frame.battery_cell_voltage3 = battery.voltage_cell3;
                frame.battery_cell_voltage4 = battery.voltage_cell4;

                let max_voltage = frame
                    .battery_cell_voltage1
                    .max(frame.battery_cell_voltage2)
                    .max(frame.battery_cell_voltage3)
                    .max(frame.battery_cell_voltage4);

                let mut min_voltage = 0.0;

                if frame.battery_cell_voltage1 > f32::default() {
                    min_voltage = frame.battery_cell_voltage1
                }
                if frame.battery_cell_voltage2 > f32::default() {
                    min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                }
                if frame.battery_cell_voltage3 > f32::default() {
                    min_voltage = min_voltage.min(frame.battery_cell_voltage3);
                }
                if frame.battery_cell_voltage4 > f32::default() {
                    min_voltage = min_voltage.min(frame.battery_cell_voltage4);
                }

                frame.battery_cell_voltage_deviation =
                    ((max_voltage - min_voltage) * 1000.0).round() / 1000.0;

                if frame.battery_cell_voltage_deviation > frame.battery_max_cell_voltage_deviation {
                    frame.battery_max_cell_voltage_deviation = frame.battery_cell_voltage_deviation;
                }

                frame.battery_temperature = battery.temperature;

                if frame.battery_temperature > frame.battery_max_temperature {
                    frame.battery_max_temperature = frame.battery_temperature
                }

                if frame.battery_temperature < frame.battery_min_temperature
                    || frame.battery_min_temperature == f32::default()
                {
                    frame.battery_min_temperature = frame.battery_temperature
                }
            }
            Record::SmartBattery(battery) => {
                frame.battery_charge_level = battery.percent;
                frame.battery_voltage = battery.voltage;
            }
            Record::SmartBatteryGroup(battery_group) => match battery_group {
                SmartBatteryGroup::SmartBatteryStatic(_) => {}
                SmartBatteryGroup::SmartBatteryDynamic(battery) => {
                    frame.battery_voltage = battery.current_voltage;
                    frame.battery_current = battery.current_current;
                    frame.battery_current_capacity = battery.remained_capacity;
                    frame.battery_full_capacity = battery.full_capacity;
                    frame.battery_charge_level = battery.capacity_percent;
                    frame.battery_temperature = battery.temperature;

                    if frame.battery_temperature > frame.battery_max_temperature {
                        frame.battery_max_temperature = frame.battery_temperature
                    }

                    if frame.battery_temperature < frame.battery_min_temperature
                        || frame.battery_min_temperature == f32::default()
                    {
                        frame.battery_min_temperature = frame.battery_temperature
                    }
                }
                SmartBatteryGroup::SmartBatterySingleVoltage(battery) => {
                    if battery.cell_count > 0 && battery.cell_voltages.len() > 0 {
                        frame.battery_cell_voltage1 = battery.cell_voltages[0];
                    }
                    if battery.cell_count > 1 && battery.cell_voltages.len() > 1 {
                        frame.battery_cell_voltage2 = battery.cell_voltages[1];
                    }
                    if battery.cell_count > 2 && battery.cell_voltages.len() > 2 {
                        frame.battery_cell_voltage3 = battery.cell_voltages[2];
                    }
                    if battery.cell_count > 3 && battery.cell_voltages.len() > 3 {
                        frame.battery_cell_voltage4 = battery.cell_voltages[3];
                    }

                    let max_voltage = frame
                        .battery_cell_voltage1
                        .max(frame.battery_cell_voltage2)
                        .max(frame.battery_cell_voltage3)
                        .max(frame.battery_cell_voltage4);

                    let mut min_voltage = 0.0;

                    if frame.battery_cell_voltage1 > f32::default() {
                        min_voltage = frame.battery_cell_voltage1
                    }
                    if frame.battery_cell_voltage2 > f32::default() {
                        min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                    }
                    if frame.battery_cell_voltage3 > f32::default() {
                        min_voltage = min_voltage.min(frame.battery_cell_voltage3);
                    }
                    if frame.battery_cell_voltage4 > f32::default() {
                        min_voltage = min_voltage.min(frame.battery_cell_voltage4);
                    }

                    frame.battery_cell_voltage_deviation =
                        ((max_voltage - min_voltage) * 1000.0).round() / 1000.0;

                    if frame.battery_cell_voltage_deviation
                        > frame.battery_max_cell_voltage_deviation
                    {
                        frame.battery_max_cell_voltage_deviation =
                            frame.battery_cell_voltage_deviation;
                    }
                }
            },
            Record::OFDM(ofdm) => {
                if ofdm.is_up {
                    frame.rc_downlink_signal = Some(ofdm.signal_percent);
                } else {
                    frame.rc_downlink_signal = Some(ofdm.signal_percent);
                }
            }
            Record::Custom(custom) => {
                frame.custom_date_time = custom.update_time_stamp;
            }
            Record::Home(home) => {
                frame.home_latitude = home.latitude;
                frame.home_longitude = home.longitude;
                // If home_altitude was not previously set, also update osd_altitude
                if frame.home_altitude == f32::default() {
                    frame.osd_altitude = frame.osd_altitude + home.altitude;
                }
                frame.home_altitude = home.altitude;
                frame.home_height_limit = home.max_allowed_height;
                frame.home_is_home_record = home.is_home_record;
                frame.home_go_home_mode = Some(home.go_home_mode);
                frame.home_is_dynamic_home_point_enabled = home.is_dynamic_home_point_enabled;
                frame.home_is_near_distance_limit = home.is_near_distance_limit;
                frame.home_is_near_height_limit = home.is_near_height_limit;
                frame.home_is_compass_calibrating = home.is_compass_adjust;
                if home.is_compass_adjust {
                    frame.home_compass_calibration_state = Some(home.compass_state);
                }
                frame.home_is_multiple_mode_enabled = home.is_multiple_mode_open;
                frame.home_is_beginner_mode = home.is_beginner_mode;
                frame.home_is_ioc_enabled = home.is_ioc_open;
                if home.is_ioc_open {
                    frame.home_ioc_mode = Some(home.ioc_mode);
                }
                frame.home_go_home_height = home.go_home_height;
                if home.is_ioc_open {
                    frame.home_ioc_course_lock_angle = Some(home.ioc_course_lock_angle);
                }
                frame.home_max_allowed_height = home.max_allowed_height;
                frame.home_current_flight_record_index = home.current_flight_record_index;
            }
            Record::Recover(recover) => {
                frame.recover_app_platform = Some(recover.app_platform);
                frame.recover_app_version = recover.app_version;
                frame.recover_aircraft_name = recover.aircraft_name;
                frame.recover_aircraft_sn = recover.aircraft_sn;
                frame.recover_camera_sn = recover.camera_sn;
                frame.recover_rc_sn = recover.rc_sn;
                frame.recover_battery_sn = recover.battery_sn;
            }
            Record::AppTip(app_tip) => {
                frame.app_tip = append_message(frame.app_tip, app_tip.message);
            }
            Record::AppWarn(app_warn) => {
                frame.app_warn = append_message(frame.app_warn, app_warn.message);
            }
            Record::AppSeriousWarn(app_serious_warn) => {
                frame.app_warn = append_message(frame.app_warn, app_serious_warn.message);
            }
            _ => {}
        }
    }

    frames
}
