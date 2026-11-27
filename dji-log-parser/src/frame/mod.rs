use serde::Serialize;
use std::borrow::Borrow;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::layout::details::Details;
use crate::record::flight_controller_common_osd_field::FlightControllerCommonOSDField;
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
mod rtk;
mod serial;
mod vision;

pub use app::FrameApp;
pub use battery::{FrameBattery, FrameBatteryPack};
pub use camera::FrameCamera;
pub use custom::FrameCustom;
pub use details::FrameDetails;
pub use gimbal::FrameGimbal;
pub use home::FrameHome;
pub use osd::FrameOSD;
pub use rc::FrameRC;
use rc::{normalized_stick, signal_percent};
pub use recover::FrameRecover;
pub use rtk::FrameRTK;
pub use serial::FrameSerial;
pub use vision::FrameVision;

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
    /// Values of all batteries: their total on drones with several batteries
    pub battery: FrameBattery,
    /// First battery of the drones with several batteries
    pub battery1: FrameBatteryPack,
    /// Second battery of the drones with several batteries
    pub battery2: FrameBatteryPack,
    pub home: FrameHome,
    pub recover: FrameRecover,
    pub app: FrameApp,
    pub vision: FrameVision,
    pub rtk: FrameRTK,
    pub serial: FrameSerial,
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
/// - `records`: The `Record` objects representing the raw log data, owned or borrowed
///   (e.g. `Vec<Record>` or `&[Record]`), so that records can be kept after conversion.
///
/// # Returns
/// - `Vec<Frame>`: A vector of `Frame` objects representing the normalized log data.
///   Each `Frame` corresponds to one or more `Record` objects, depending on the
///   specific normalization logic.
///
pub fn records_to_frames<I>(records: I, details: Details) -> Vec<Frame>
where
    I: IntoIterator,
    I::Item: Borrow<Record>,
{
    Frames::new(records.into_iter(), details).collect()
}

/// Iterator over the frames built from records, see
/// [`DJILog::frames_iter`](crate::DJILog::frames_iter).
pub struct Frames<I> {
    records: I,
    /// None once the last frame is returned
    builder: Option<FrameBuilder>,
}

impl<I> Frames<I> {
    pub(crate) fn new(records: I, details: Details) -> Self {
        Frames {
            records,
            builder: Some(FrameBuilder::new(&details)),
        }
    }
}

impl<I> Iterator for Frames<I>
where
    I: Iterator,
    I::Item: Borrow<Record>,
{
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        let builder = self.builder.as_mut()?;
        for record in self.records.by_ref() {
            if let Some(frame) = builder.push(record.borrow()) {
                return Some(frame);
            }
        }
        self.builder.take()?.finish()
    }
}

/// Builds the frames from the records, one record at a time.
struct FrameBuilder {
    frame: Frame,
    frame_index: usize,
    /// On drones with several batteries, the battery of index 0 is the total of all batteries
    has_total_battery: bool,
    /// The camera records keep their photo flag for several records
    is_shooting_photo: bool,
    /// Serial numbers of the serial number records
    serials: FrameSerial,
    /// Serial numbers of the details and of the recover records, used for the components without
    /// serial number records
    fallback_serials: FrameSerial,
}

impl FrameBuilder {
    fn new(details: &Details) -> Self {
        FrameBuilder {
            frame: Frame {
                battery: FrameBattery {
                    cell_num: details.product_type.battery_cell_num(),
                    cell_voltages: vec![0.0; details.product_type.battery_cell_num() as usize],
                    is_cell_voltage_estimated: true,
                    ..FrameBattery::default()
                },
                serial: FrameSerial::from_details(details),
                ..Frame::default()
            },
            frame_index: 0,
            has_total_battery: false,
            is_shooting_photo: false,
            serials: FrameSerial::default(),
            fallback_serials: FrameSerial::from_details(details),
        }
    }

    /// Adds a record to the current frame. Returns the previous frame when an OSD record starts a
    /// new one.
    fn push(&mut self, record: &Record) -> Option<Frame> {
        let mut completed = None;
        let frame = &mut self.frame;
        match record {
            Record::OSD(osd) => {
                if self.frame_index > 0 {
                    frame.finalize();
                    completed = Some(frame.clone());
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
                    append_message(
                        &mut frame.app.tip,
                        &format!("Flight mode changed to {:?}.", osd.flight_mode),
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
                frame.osd.is_gps_used = osd.is_gps_valid;
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
                frame.osd.is_accelerometer_over_range = osd.is_accelerometer_over_range;
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
                frame.osd.wind_speed = osd.wind_speed;
                frame.osd.wind_direction = osd.wind_direction;
                frame.osd.position_source = osd.position_source;
                frame.battery.low_warning = Some(osd.low_warning);
                frame.battery.serious_low_warning = Some(osd.serious_low_warning);

                self.frame_index += 1;
            }
            Record::Gimbal(gimbal) => {
                frame.gimbal.mode = Some(gimbal.mode);
                frame.gimbal.pitch = gimbal.pitch;
                frame.gimbal.roll = gimbal.roll;
                frame.gimbal.yaw = gimbal.yaw;
                if !frame.gimbal.is_pitch_at_limit && gimbal.is_pitch_at_limit {
                    append_message(&mut frame.app.tip, "Gimbal pitch axis endpoint reached.")
                }
                frame.gimbal.is_pitch_at_limit = gimbal.is_pitch_at_limit;
                if !frame.gimbal.is_roll_at_limit && gimbal.is_roll_at_limit {
                    append_message(&mut frame.app.tip, "Gimbal roll axis endpoint reached.")
                }
                frame.gimbal.is_roll_at_limit = gimbal.is_roll_at_limit;
                if !frame.gimbal.is_yaw_at_limit && gimbal.is_yaw_at_limit {
                    append_message(&mut frame.app.tip, "Gimbal yaw axis endpoint reached.")
                }
                frame.gimbal.is_yaw_at_limit = gimbal.is_yaw_at_limit;
                frame.gimbal.is_stuck = gimbal.is_stuck;
            }
            Record::Camera(camera) => {
                if camera.is_shooting_single_photo && !self.is_shooting_photo {
                    frame.camera.is_photo = true;
                }
                self.is_shooting_photo = camera.is_shooting_single_photo;
                frame.camera.is_video = camera.is_recording;
                frame.camera.sd_card_is_inserted = camera.has_sd_card;
                frame.camera.sd_card_state = Some(camera.sd_card_state);
            }
            Record::RC(rc) => {
                frame
                    .rc
                    .set_sticks(rc.aileron, rc.elevator, rc.throttle, rc.rudder);
            }
            Record::RCDisplayField(rc) => {
                frame
                    .rc
                    .set_sticks(rc.aileron, rc.elevator, rc.throttle, rc.rudder);
                frame.rc.mode_switch = Some(rc.mode_switch);
            }
            Record::AppJoystick(joystick) => {
                frame.app.aileron = Some(joystick.aileron);
                frame.app.elevator = Some(joystick.elevator);
                frame.app.throttle = Some(joystick.throttle);
                frame.app.rudder = Some(joystick.rudder);
                frame.app.aileron_normalized = Some(normalized_stick(joystick.aileron));
                frame.app.elevator_normalized = Some(normalized_stick(joystick.elevator));
                frame.app.throttle_normalized = Some(normalized_stick(joystick.throttle));
                frame.app.rudder_normalized = Some(normalized_stick(joystick.rudder));
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
                frame.battery.useful_time = Some(battery.useful_time);
                frame.battery.go_home_time = Some(battery.go_home_time);
                frame.battery.land_time = Some(battery.land_time);
                frame.battery.go_home_battery = Some(battery.go_home_battery);
                frame.battery.land_battery = Some(battery.land_battery);
            }
            Record::FlightControllerCommonOSDField(FlightControllerCommonOSDField::BatteryOsd(
                battery,
            )) => {
                frame.battery.useful_time = Some(battery.useful_time);
                frame.battery.go_home_time = battery.go_home_time;
                frame.battery.land_time = Some(battery.land_time);
                frame.battery.go_home_battery = Some(battery.go_home_battery.into());
                frame.battery.land_battery = Some(battery.land_battery.into());
            }
            Record::SmartBatteryGroup(battery_group) => match battery_group {
                SmartBatteryGroup::SmartBatteryStatic(_) => {}
                SmartBatteryGroup::SmartBatteryDynamic(battery) => {
                    self.has_total_battery |= battery.index == 0;
                    match battery.index {
                        1 => frame.battery1.update(battery),
                        2 => frame.battery2.update(battery),
                        _ => {}
                    }
                    // The first battery stands for all of them in logs without a total
                    if battery.index == 0 || (battery.index == 1 && !self.has_total_battery) {
                        frame.battery.voltage = battery.current_voltage;
                        frame.battery.current = battery.current_current;
                        frame.battery.current_capacity = battery.remained_capacity;
                        frame.battery.full_capacity = battery.full_capacity;
                        frame.battery.charge_level = battery.capacity_percent;
                        frame.battery.temperature = battery.temperature;
                    }
                }
                // The cell voltages of one battery only
                SmartBatteryGroup::SmartBatterySingleVoltage(battery) if battery.index <= 1 => {
                    let cell_num = frame
                        .battery
                        .cell_voltages
                        .len()
                        .min(battery.cell_count as usize);

                    frame.battery.is_cell_voltage_estimated = false;

                    frame.battery.cell_voltages[..cell_num]
                        .copy_from_slice(&battery.cell_voltages[..cell_num]);
                }
                SmartBatteryGroup::SmartBatterySingleVoltage(_) => {}
            },
            Record::VisionGroup(group) => {
                for entry in &group.entries {
                    frame.vision.update(entry);
                }
            }
            Record::OFDM(ofdm) => {
                let signal = Some(signal_percent(ofdm.signal, frame.osd.drone_type));
                if ofdm.is_up {
                    frame.rc.uplink_signal = signal;
                } else {
                    frame.rc.downlink_signal = signal;
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
            Record::ComponentSerial(component) => {
                self.serials.update(component);
                frame.serial = self.serials.or(&self.fallback_serials);
            }
            Record::Recover(recover) => {
                self.fallback_serials.update_from_recover(recover);
                frame.serial = self.serials.or(&self.fallback_serials);

                frame.recover.app_platform = Some(recover.app_platform.clone());
                frame.recover.app_version = recover.app_version.clone();
                frame.recover.aircraft_name = recover.aircraft_name.clone();
                frame.recover.aircraft_sn = recover.aircraft_sn.clone();
                frame.recover.camera_sn = recover.camera_sn.clone();
                frame.recover.rc_sn = recover.rc_sn.clone();
                frame.recover.battery_sn = recover.battery_sn.clone();
            }
            Record::RTKDifference(rtk) => frame.rtk.update(rtk),
            Record::GOBusiness(business) => {
                for tip in business.tips.iter().flatten() {
                    append_message(&mut frame.app.tip, tip.text.as_str());
                }
                for warning in business.warnings.iter().flatten() {
                    if warning.level.unwrap_or_default() >= 2 {
                        append_message(&mut frame.app.warn, warning.text.as_str());
                    } else {
                        append_message(&mut frame.app.tip, warning.text.as_str());
                    }
                }
            }
            Record::AppTip(app_tip) => {
                append_message(&mut frame.app.tip, app_tip.message.as_str());
            }
            Record::AppWarn(app_warn) => {
                append_message(&mut frame.app.warn, app_warn.message.as_str());
            }
            Record::AppSeriousWarn(app_serious_warn) => {
                append_message(&mut frame.app.warn, app_serious_warn.message.as_str());
            }
            _ => {}
        }
        completed
    }

    /// Returns the last frame: a frame is only returned when the next OSD record starts a new one.
    fn finish(mut self) -> Option<Frame> {
        (self.frame_index > 0).then(|| {
            self.frame.finalize();
            self.frame
        })
    }
}
