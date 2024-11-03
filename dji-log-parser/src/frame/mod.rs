use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::layout::details::Details;
use crate::record::osd::{AppCommand, GroundOrSky};
use crate::record::smart_battery_group::SmartBatteryGroup;
use crate::record::Record;
use crate::utils::append_message;

mod app;
mod battery;
mod camera;
mod custom;
mod details;
mod gimbal;
mod home;
mod osd;
mod rc;
mod recover;

pub use app::FrameApp;
pub use battery::FrameBattery;
pub use camera::FrameCamera;
pub use custom::FrameCustom;
pub use details::FrameDetails;
pub use gimbal::FrameGimbal;
pub use home::FrameHome;
pub use osd::FrameOSD;
pub use rc::FrameRC;
pub use recover::FrameRecover;

/// Represents a normalized frame of data from a DJI log.
///
/// A `Frame` is a standardized representation of log data, normalized across
/// different log versions. It provides a consistent and easy-to-use format
/// for analyzing and processing DJI log information.
///
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Frame {
    pub custom: FrameCustom,
    pub osd: FrameOSD,
    pub gimbal: FrameGimbal,
    pub camera: FrameCamera,
    pub rc: FrameRC,
    pub battery: FrameBattery,
    pub home: FrameHome,
    pub recover: FrameRecover,
    pub app: FrameApp,
}

impl Frame {
    /// Resets event-related values of the `Frame` instance.
    ///
    /// This method resets the state of the camera, application tips, and warnings.
    /// Additionally, if the battery cell voltage is estimated, it resets all cell voltages to zero.
    ///
    fn reset(&mut self) {
        self.camera.is_photo = bool::default();
        self.app.tip = String::default();
        self.app.warn = String::default();

        if self.battery.is_cell_voltage_estimated {
            self.battery.cell_voltages.fill(0.0);
        }
    }

    /// Computes derived values based on the current state of the `Frame` instance.
    ///
    /// This method finalizes the state of the `Frame` by computing any values that are
    /// derived from the current attributes. This is typically called after all primary
    /// attributes have been set or modified.
    ///
    fn finalize(&mut self) {
        if self.osd.height_max < self.osd.height {
            self.osd.height_max = self.osd.height;
        }
        if self.osd.x_speed_max < self.osd.x_speed {
            self.osd.x_speed_max = self.osd.x_speed;
        }
        if self.osd.y_speed_max < self.osd.y_speed {
            self.osd.y_speed_max = self.osd.y_speed;
        }
        if self.osd.z_speed_max < self.osd.z_speed {
            self.osd.z_speed_max = self.osd.z_speed;
        }

        if let Some(first_cell) = self.battery.cell_voltages.first() {
            if *first_cell == 0.0 && self.battery.voltage > 0.0 {
                self.battery.is_cell_voltage_estimated = true;
                self.battery
                    .cell_voltages
                    .fill(self.battery.voltage / self.battery.cell_num as f32)
            }
        }

        if self.battery.temperature > self.battery.max_temperature {
            self.battery.max_temperature = self.battery.temperature
        }

        if self.battery.temperature < self.battery.min_temperature
            || self.battery.min_temperature == f32::default()
        {
            self.battery.min_temperature = self.battery.temperature
        }

        let max_voltage = self
            .battery
            .cell_voltages
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        let min_voltage = self
            .battery
            .cell_voltages
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        self.battery.cell_voltage_deviation =
            ((max_voltage - min_voltage) * 1000.0).round() / 1000.0;

        if self.battery.cell_voltage_deviation > self.battery.max_cell_voltage_deviation {
            self.battery.max_cell_voltage_deviation = self.battery.cell_voltage_deviation;
        }
    }
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
    let mut frame = Frame {
        battery: FrameBattery {
            cell_num: details.product_type.battery_cell_num(),
            cell_voltages: vec![0.0; details.product_type.battery_cell_num() as usize],
            is_cell_voltage_estimated: true,
            ..FrameBattery::default()
        },
        ..Frame::default()
    };

    let mut frame_index = 0;

    for record in records {
        match record {
            Record::OSD(osd) => {
                if frame_index > 0 {
                    frame.finalize();
                    frames.push(frame.clone());
                    frame.reset();
                }

                // Fill OSD record
                frame.osd.fly_time = osd.fly_time;
                frame.osd.latitude = osd.latitude;
                frame.osd.longitude = osd.longitude;
                // Fix altitude by adding the home point altitude
                frame.osd.altitude = osd.altitude + frame.home.altitude;
                frame.osd.height = osd.altitude;
                frame.osd.vps_height = osd.s_wave_height;

                frame.osd.x_speed = osd.speed_x;
                frame.osd.y_speed = osd.speed_y;
                frame.osd.z_speed = osd.speed_z;
                frame.osd.pitch = osd.pitch;
                frame.osd.yaw = osd.yaw;
                frame.osd.roll = osd.roll;

                if frame.osd.flyc_state != Some(osd.flight_mode) {
                    frame.app.tip = append_message(
                        frame.app.tip,
                        format!("Flight mode changed to {:?}.", osd.flight_mode),
                    );
                }
                frame.osd.flyc_state = Some(osd.flight_mode);
                if let AppCommand::Unknown(0) = osd.app_command {
                    frame.osd.flyc_command = None;
                } else {
                    frame.osd.flyc_command = Some(osd.app_command);
                }
                frame.osd.flight_action = Some(osd.flight_action);
                frame.osd.gps_num = osd.gps_num;
                frame.osd.gps_level = osd.gps_level;
                frame.osd.is_gpd_used = osd.is_gps_valid;
                frame.osd.non_gps_cause = Some(osd.non_gps_cause);
                frame.osd.drone_type = Some(osd.drone_type);
                frame.osd.is_swave_work = osd.is_swave_work;
                frame.osd.wave_error = osd.wave_error;
                frame.osd.go_home_status = Some(osd.go_home_status);
                frame.osd.battery_type = Some(osd.battery_type);
                frame.osd.is_on_ground = osd.ground_or_sky == GroundOrSky::Ground;
                frame.osd.is_motor_on = osd.is_motor_up;
                frame.osd.is_motor_blocked = osd.is_motor_blocked;
                frame.osd.motor_start_failed_cause = Some(osd.motor_start_failed_cause);
                frame.osd.is_imu_preheated = osd.is_imu_preheated;
                frame.osd.imu_init_fail_reason = Some(osd.imu_init_fail_reason);
                frame.osd.is_acceletor_over_range = osd.is_acceletor_over_range;
                frame.osd.is_barometer_dead_in_air = osd.is_barometer_dead_in_air;
                frame.osd.is_compass_error = osd.is_compass_error;
                frame.osd.is_go_home_height_modified = osd.is_go_home_height_modified;
                frame.osd.can_ioc_work = osd.can_ioc_work;
                frame.osd.is_not_enough_force = osd.is_not_enough_force;
                frame.osd.is_out_of_limit = osd.is_out_of_limit;
                frame.osd.is_propeller_catapult = osd.is_propeller_catapult;
                frame.osd.is_vibrating = osd.is_vibrating;
                frame.osd.is_vision_used = osd.is_vision_used;
                frame.osd.voltage_warning = osd.voltage_warning;

                frame_index += 1;
            }
            Record::Gimbal(gimbal) => {
                frame.gimbal.mode = Some(gimbal.mode);
                frame.gimbal.pitch = gimbal.pitch;
                frame.gimbal.roll = gimbal.roll;
                frame.gimbal.yaw = gimbal.yaw;
                if !frame.gimbal.is_pitch_at_limit && gimbal.is_pitch_at_limit {
                    frame.app.tip =
                        append_message(frame.app.tip, "Gimbal pitch axis endpoint reached.")
                }
                frame.gimbal.is_pitch_at_limit = gimbal.is_pitch_at_limit;
                if !frame.gimbal.is_roll_at_limit && gimbal.is_roll_at_limit {
                    frame.app.tip =
                        append_message(frame.app.tip, "Gimbal roll axis endpoint reached.")
                }
                frame.gimbal.is_roll_at_limit = gimbal.is_roll_at_limit;
                if !frame.gimbal.is_yaw_at_limit && gimbal.is_yaw_at_limit {
                    frame.app.tip =
                        append_message(frame.app.tip, "Gimbal yaw axis endpoint reached.")
                }
                frame.gimbal.is_yaw_at_limit = gimbal.is_yaw_at_limit;
                frame.gimbal.is_stuck = gimbal.is_stuck;
            }
            Record::Camera(camera) => {
                frame.camera.is_photo = camera.is_shooting_single_photo;
                frame.camera.is_video = camera.is_recording;
                frame.camera.sd_card_is_inserted = camera.has_sd_card;
                frame.camera.sd_card_state = Some(camera.sd_card_state);
            }
            Record::RC(rc) => {
                frame.rc.aileron = rc.aileron;
                frame.rc.elevator = rc.elevator;
                frame.rc.throttle = rc.throttle;
                frame.rc.rudder = rc.rudder;
            }
            Record::RCDisplayField(rc) => {
                frame.rc.aileron = rc.aileron;
                frame.rc.elevator = rc.elevator;
                frame.rc.throttle = rc.throttle;
                frame.rc.rudder = rc.rudder;
            }
            Record::CenterBattery(battery) => {
                frame.battery.charge_level = battery.relative_capacity;
                frame.battery.voltage = battery.voltage;
                frame.battery.current_capacity = battery.current_capacity as u32;
                frame.battery.full_capacity = battery.full_capacity as u32;
                frame.battery.full_capacity = battery.full_capacity as u32;
                frame.battery.is_cell_voltage_estimated = false;

                let cell_num = frame.battery.cell_voltages.len();
                if cell_num > 0 {
                    frame.battery.cell_voltages[0] = battery.voltage_cell1;
                }
                if cell_num > 1 {
                    frame.battery.cell_voltages[1] = battery.voltage_cell2;
                }
                if cell_num > 2 {
                    frame.battery.cell_voltages[2] = battery.voltage_cell3;
                }
                if cell_num > 3 {
                    frame.battery.cell_voltages[3] = battery.voltage_cell4;
                }
                if cell_num > 4 {
                    frame.battery.cell_voltages[4] = battery.voltage_cell5;
                }
                if cell_num > 5 {
                    frame.battery.cell_voltages[5] = battery.voltage_cell6;
                }
            }
            Record::SmartBattery(battery) => {
                frame.battery.charge_level = battery.percent;
                frame.battery.voltage = battery.voltage;
            }
            Record::SmartBatteryGroup(battery_group) => match battery_group {
                SmartBatteryGroup::SmartBatteryStatic(_) => {}
                SmartBatteryGroup::SmartBatteryDynamic(battery) => {
                    // when there are multiple batteries, only one contains accurate values at index 1
                    if details.product_type.battery_num() < 2 || battery.index == 1 {
                        frame.battery.voltage = battery.current_voltage;
                        frame.battery.current = battery.current_current;
                        frame.battery.current_capacity = battery.remained_capacity;
                        frame.battery.full_capacity = battery.full_capacity;
                        frame.battery.charge_level = battery.capacity_percent;
                        frame.battery.temperature = battery.temperature;
                    }
                }
                SmartBatteryGroup::SmartBatterySingleVoltage(battery) => {
                    let cell_num = frame
                        .battery
                        .cell_voltages
                        .len()
                        .min(battery.cell_count as usize);

                    frame.battery.is_cell_voltage_estimated = false;

                    frame.battery.cell_voltages[..cell_num]
                        .copy_from_slice(&battery.cell_voltages[..cell_num]);
                }
            },
            Record::OFDM(ofdm) => {
                if ofdm.is_up {
                    frame.rc.uplink_signal = Some(ofdm.signal_percent);
                } else {
                    frame.rc.downlink_signal = Some(ofdm.signal_percent);
                }
            }
            Record::Custom(custom) => {
                frame.custom.date_time = custom.update_timestamp;
            }
            Record::Home(home) => {
                frame.home.latitude = home.latitude;
                frame.home.longitude = home.longitude;
                // If home_altitude was not previously set, also update osd.altitude
                if frame.home.altitude == f32::default() {
                    frame.osd.altitude += home.altitude;
                }
                frame.home.altitude = home.altitude;
                frame.home.height_limit = home.max_allowed_height;
                frame.home.is_home_record = home.is_home_record;
                frame.home.go_home_mode = Some(home.go_home_mode);
                frame.home.is_dynamic_home_point_enabled = home.is_dynamic_home_point_enabled;
                frame.home.is_near_distance_limit = home.is_near_distance_limit;
                frame.home.is_near_height_limit = home.is_near_height_limit;
                frame.home.is_compass_calibrating = home.is_compass_adjust;
                if home.is_compass_adjust {
                    frame.home.compass_calibration_state = Some(home.compass_state);
                }
                frame.home.is_multiple_mode_enabled = home.is_multiple_mode_open;
                frame.home.is_beginner_mode = home.is_beginner_mode;
                frame.home.is_ioc_enabled = home.is_ioc_open;
                if home.is_ioc_open {
                    frame.home.ioc_mode = Some(home.ioc_mode);
                }
                frame.home.go_home_height = home.go_home_height;
                if home.is_ioc_open {
                    frame.home.ioc_course_lock_angle = Some(home.ioc_course_lock_angle);
                }
                frame.home.max_allowed_height = home.max_allowed_height;
                frame.home.current_flight_record_index = home.current_flight_record_index;
            }
            Record::Recover(recover) => {
                frame.recover.app_platform = Some(recover.app_platform);
                frame.recover.app_version = recover.app_version;
                frame.recover.aircraft_name = recover.aircraft_name;
                frame.recover.aircraft_sn = recover.aircraft_sn;
                frame.recover.camera_sn = recover.camera_sn;
                frame.recover.rc_sn = recover.rc_sn;
                frame.recover.battery_sn = recover.battery_sn;
            }
            Record::AppTip(app_tip) => {
                frame.app.tip = append_message(frame.app.tip, app_tip.message);
            }
            Record::AppWarn(app_warn) => {
                frame.app.warn = append_message(frame.app.warn, app_warn.message);
            }
            Record::AppSeriousWarn(app_serious_warn) => {
                frame.app.warn = append_message(frame.app.warn, app_serious_warn.message);
            }
            _ => {}
        }
    }

    frames
}
