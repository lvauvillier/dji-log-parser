use chrono::{DateTime, Utc};
use csv::Writer;
use dji_log_parser::record::camera::SDCardState;
use dji_log_parser::record::gimbal::GimbalMode;
use dji_log_parser::record::home::{CompassCalibrationState, GoHomeMode, IOCMode};
use dji_log_parser::record::osd::{
    AppCommand, BatteryType, DroneType, FlightAction, FlightMode, GoHomeStatus, GroundOrSky,
    ImuInitFailReason, MotorStartFailedCause, NonGPSCause,
};

use dji_log_parser::record::smart_battery_group::SmartBatteryGroup;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use serde::Serialize;

use crate::{Cli, Exporter};

#[derive(Serialize, Debug, Default, Clone)]
pub struct Frame {
    #[serde(rename = "CUSTOM.dateTime")]
    pub custom_date_time: DateTime<Utc>,

    /// seconds
    #[serde(rename = "OSD.flyTime")]
    pub osd_fly_time: f32,
    /// degrees
    #[serde(rename = "OSD.lalitude")]
    pub osd_latitude: f64,
    /// degrees
    #[serde(rename = "OSD.longitude")]
    pub osd_longitude: f64,
    /// meters
    #[serde(rename = "OSD.height")]
    pub osd_height: f32,
    /// meters
    #[serde(rename = "OSD.heightMax")]
    pub osd_height_max: f32,
    /// meters
    #[serde(rename = "OSD.vpsHeight")]
    pub osd_vps_height: f32,
    /// meters
    #[serde(rename = "OSD.altitude")]
    pub osd_altitude: f32,
    /// meters / second
    #[serde(rename = "OSD.xSpeed")]
    pub osd_x_speed: f32,
    /// meters / second
    #[serde(rename = "OSD.xSpeedMax")]
    pub osd_x_speed_max: f32,
    /// meters / second
    #[serde(rename = "OSD.ySpeed")]
    pub osd_y_speed: f32,
    /// meters / second
    #[serde(rename = "OSD.ySpeedMax")]
    pub osd_y_speed_max: f32,
    /// meters / second
    #[serde(rename = "OSD.zSpeed")]
    pub osd_z_speed: f32,
    /// meters / second
    #[serde(rename = "OSD.zSpeedMax")]
    pub osd_z_speed_max: f32,
    /// degrees
    #[serde(rename = "OSD.pitch")]
    pub osd_pitch: f32,
    /// degrees
    #[serde(rename = "OSD.roll")]
    pub osd_roll: f32,
    /// degrees
    #[serde(rename = "OSD.yaw")]
    pub osd_yaw: f32,
    #[serde(rename = "OSD.flycState")]
    pub osd_flyc_state: Option<FlightMode>,
    #[serde(rename = "OSD.flycCommand")]
    pub osd_flyc_command: Option<AppCommand>,
    #[serde(rename = "OSD.flightAction")]
    pub osd_flight_action: Option<FlightAction>,
    #[serde(rename = "OSD.isGPSUsed")]
    pub osd_is_gpd_used: bool,
    #[serde(rename = "OSD.nonGPSCause")]
    pub osd_non_gps_cause: Option<NonGPSCause>,
    #[serde(rename = "OSD.gpsNum")]
    pub osd_gps_num: u8,
    #[serde(rename = "OSD.gpsLevel")]
    pub osd_gps_level: u8,
    #[serde(rename = "OSD.droneType")]
    pub osd_drone_type: Option<DroneType>,
    #[serde(rename = "OSD.isSwaveWork")]
    pub osd_is_swave_work: bool,
    #[serde(rename = "OSD.waveError")]
    pub osd_wave_error: bool,
    #[serde(rename = "OSD.goHomeStatus")]
    pub osd_go_home_status: Option<GoHomeStatus>,
    #[serde(rename = "OSD.batteryType")]
    pub osd_battery_type: Option<BatteryType>,
    #[serde(rename = "OSD.isOnGround")]
    pub osd_is_on_ground: bool,
    #[serde(rename = "OSD.isMotorOn")]
    pub osd_is_motor_on: bool,
    #[serde(rename = "OSD.isMotorBlocked")]
    pub osd_is_motor_blocked: bool,
    #[serde(rename = "OSD.motorStartFailedCause")]
    pub osd_motor_start_failed_cause: Option<MotorStartFailedCause>,
    #[serde(rename = "OSD.isImuPreheated")]
    pub osd_is_imu_preheated: bool,
    #[serde(rename = "OSD.imuInitFailReason")]
    pub osd_imu_init_fail_reason: Option<ImuInitFailReason>,
    #[serde(rename = "OSD.isAcceleratorOverRange")]
    pub osd_is_acceletor_over_range: bool,
    #[serde(rename = "OSD.isBarometerDeadInAir")]
    pub osd_is_barometer_dead_in_air: bool,
    #[serde(rename = "OSD.isCompassError")]
    pub osd_is_compass_error: bool,
    #[serde(rename = "OSD.isGoHomeHeightModified")]
    pub osd_is_go_home_height_modified: bool,
    #[serde(rename = "OSD.canIOCWork")]
    pub osd_can_ioc_work: bool,
    #[serde(rename = "OSD.isNotEnoughForce")]
    pub osd_is_not_enough_force: bool,
    #[serde(rename = "OSD.isOutOfLimit")]
    pub osd_is_out_of_limit: bool,
    #[serde(rename = "OSD.isPropellerCatapult")]
    pub osd_is_propeller_catapult: bool,
    #[serde(rename = "OSD.isVibrating")]
    pub osd_is_vibrating: bool,
    #[serde(rename = "OSD.isVisionUsed")]
    pub osd_is_vision_used: bool,
    #[serde(rename = "OSD.voltageWarning")]
    pub osd_voltage_warning: u8,

    #[serde(rename = "GIMBAL.mode")]
    pub gimbal_mode: Option<GimbalMode>,
    /// degrees
    #[serde(rename = "GIMBAL.pitch")]
    pub gimbal_pitch: f32,
    /// degrees
    #[serde(rename = "GIMBAL.roll")]
    pub gimbal_roll: f32,
    /// degrees
    #[serde(rename = "GIMBAL.yaw")]
    pub gimbal_yaw: f32,
    #[serde(rename = "GIMBAL.isPitchAtLimit")]
    pub gimbal_is_pitch_at_limit: bool,
    #[serde(rename = "GIMBAL.isRollAtLimit")]
    pub gimbal_is_roll_at_limit: bool,
    #[serde(rename = "GIMBAL.isYawAtLimit")]
    pub gimbal_is_yaw_at_limit: bool,
    #[serde(rename = "GIMBAL.isStuck")]
    pub gimbal_is_stuck: bool,

    #[serde(rename = "CAMERA.isPhoto")]
    pub camera_is_photo: bool,
    #[serde(rename = "CAMERA.isVideo")]
    pub camera_is_video: bool,
    #[serde(rename = "CAMERA.sdCardIsInserted")]
    pub camera_sd_card_is_inserted: bool,
    #[serde(rename = "CAMERA.sdCardState")]
    pub camera_sd_card_state: Option<SDCardState>,

    #[serde(rename = "RC.downlinkSignal")]
    pub rc_downlink_signal: Option<u8>,
    #[serde(rename = "RC.uplinkSignal")]
    pub rc_uplink_signal: Option<u8>,
    #[serde(rename = "RC.aileron")]
    /// right stick - horizontal
    pub rc_aileron: u16,
    #[serde(rename = "RC.elevator")]
    /// right stick - vertical
    pub rc_elevator: u16,
    #[serde(rename = "RC.throttle")]
    /// left stick - vertical
    pub rc_throttle: u16,
    #[serde(rename = "RC.rudder")]
    /// left stick - horizontal
    pub rc_rudder: u16,

    #[serde(rename = "BATTERY.chargeLevel")]
    pub battery_charge_level: u8,
    #[serde(rename = "BATTERY.voltage")]
    pub battery_voltage: f32,
    #[serde(rename = "BATTERY.current")]
    pub battery_current: f32,
    #[serde(rename = "BATTERY.currentCapacity")]
    pub battery_current_capacity: u32,
    #[serde(rename = "BATTERY.fullCapacity")]
    pub battery_full_capacity: u32,
    #[serde(rename = "BATTERY.cellVoltage1")]
    pub battery_cell_voltage1: f32,
    #[serde(rename = "BATTERY.cellVoltage2")]
    pub battery_cell_voltage2: f32,
    #[serde(rename = "BATTERY.cellVoltage3")]
    pub battery_cell_voltage3: f32,
    #[serde(rename = "BATTERY.cellVoltage4")]
    pub battery_cell_voltage4: f32,
    #[serde(rename = "BATTERY.cellVoltageDeviation")]
    pub battery_cell_voltage_deviation: f32,
    #[serde(rename = "BATTERY.maxCellVoltageDeviation")]
    pub battery_max_cell_voltage_deviation: f32,
    #[serde(rename = "BATTERY.temperature")]
    pub battery_temperature: f32,
    #[serde(rename = "BATTERY.minTemperature")]
    pub battery_min_temperature: f32,
    #[serde(rename = "BATTERY.maxTemperature")]
    pub battery_max_temperature: f32,

    /// degrees
    #[serde(rename = "HOME.latitude")]
    pub home_latitude: f64,
    /// degrees
    #[serde(rename = "HOME.longitude")]
    pub home_longitude: f64,
    /// meters
    #[serde(rename = "HOME.altitude")]
    pub home_altitude: f32,
    /// meters
    /// #[serde(rename = "HOME.heightLimit")]
    /// pub home_height_limit: f32,
    #[serde(rename = "HOME.isHomeRecord")]
    pub home_is_home_record: bool,
    #[serde(rename = "HOME.goHomeMode")]
    pub home_go_home_mode: Option<GoHomeMode>,
    #[serde(rename = "HOME.isDynamicHomePointEnabled")]
    pub home_is_dynamic_home_point_enabled: bool,
    #[serde(rename = "HOME.isNearDistanceLimit")]
    pub home_is_near_distance_limit: bool,
    #[serde(rename = "HOME.isNearHeightLimit")]
    pub home_is_near_height_limit: bool,
    #[serde(rename = "HOME.isCompassCalibrating")]
    pub home_is_compass_calibrating: bool,
    #[serde(rename = "HOME.compassCalibrationState")]
    pub home_compass_calibration_state: Option<CompassCalibrationState>,
    #[serde(rename = "HOME.isMultipleModeEnabled")]
    pub home_is_multiple_mode_enabled: bool,
    #[serde(rename = "HOME.isBeginnerMode")]
    pub home_is_beginner_mode: bool,
    #[serde(rename = "HOME.isIOCEnabled")]
    pub home_is_ioc_enabled: bool,
    #[serde(rename = "HOME.IOCMode")]
    pub home_ioc_mode: Option<IOCMode>,
    /// meters
    #[serde(rename = "HOME.goHomeHeight")]
    pub home_go_home_height: u16,
    #[serde(rename = "HOME.IOCCourseLockAngle")]
    pub home_ioc_course_lock_angle: Option<i16>,
    /// meters
    #[serde(rename = "HOME.maxAllowedHeight")]
    pub home_max_allowed_height: f32,
    #[serde(rename = "HOME.currentFlightRecordIndex")]
    pub home_current_flight_record_index: u16,
}

#[derive(Default)]
pub struct CSVExporter;

impl Exporter for CSVExporter {
    fn export(&self, parser: &DJILog, records: &Vec<Record>, args: &Cli) {
        if let Some(csv_path) = &args.csv {
            let mut writer = Writer::from_path(csv_path).unwrap();

            //let mut frames = vec![];
            let mut frame = Frame::default();
            let mut frame_index = 0;

            for record in records {
                match record {
                    Record::OSD(osd) => {
                        // Push a new frame
                        if frame_index > 0 {
                            writer.serialize(&frame).unwrap();
                        }

                        // Reset non persistant values (one time events)
                        frame.camera_is_photo = bool::default();
                        frame.camera_is_video = bool::default();

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
                        frame.gimbal_is_pitch_at_limit = gimbal.is_pitch_at_limit;
                        frame.gimbal_is_roll_at_limit = gimbal.is_roll_at_limit;
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
                            .min(frame.battery_cell_voltage2)
                            .min(frame.battery_cell_voltage3)
                            .min(frame.battery_cell_voltage4);

                        let mut min_voltage = 0.0;

                        if frame.battery_cell_voltage1 > f32::default() {
                            min_voltage = frame.battery_cell_voltage1
                        }
                        if frame.battery_cell_voltage2 > f32::default() {
                            min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                        }
                        if frame.battery_cell_voltage3 > f32::default() {
                            min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                        }
                        if frame.battery_cell_voltage4 > f32::default() {
                            min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                        }

                        frame.battery_cell_voltage_deviation =
                            ((max_voltage - min_voltage) * 1000.0).round() / 1000.0;

                        if frame.battery_cell_voltage_deviation
                            > frame.battery_max_cell_voltage_deviation
                        {
                            frame.battery_max_cell_voltage_deviation =
                                frame.battery_cell_voltage_deviation;
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
                                .min(frame.battery_cell_voltage2)
                                .min(frame.battery_cell_voltage3)
                                .min(frame.battery_cell_voltage4);

                            let mut min_voltage = 0.0;

                            if frame.battery_cell_voltage1 > f32::default() {
                                min_voltage = frame.battery_cell_voltage1
                            }
                            if frame.battery_cell_voltage2 > f32::default() {
                                min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                            }
                            if frame.battery_cell_voltage3 > f32::default() {
                                min_voltage = min_voltage.min(frame.battery_cell_voltage2);
                            }
                            if frame.battery_cell_voltage4 > f32::default() {
                                min_voltage = min_voltage.min(frame.battery_cell_voltage2);
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
                        frame.home_is_home_record = home.is_home_record;
                        frame.home_go_home_mode = Some(home.go_home_mode);
                        frame.home_is_dynamic_home_point_enabled =
                            home.is_dynamic_home_point_enabled;
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
                    _ => {}
                }
            }
        }
    }
}