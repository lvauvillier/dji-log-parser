use csv::WriterBuilder;
use dji_log_parser::frame::Frame;
use dji_log_parser::frame::FrameDetails;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use serde::Serialize;

use crate::{Cli, Exporter};

#[derive(Serialize)]
struct FrameWithDetails<'a> {
    frame: &'a Frame,
    details: &'a FrameDetails,
}

#[derive(Default)]
pub struct CSVExporter;

impl Exporter for CSVExporter {
    fn export(&self, parser: &DJILog, _records: &Vec<Record>, frames: &Vec<Frame>, args: &Cli) {
        if let Some(csv_path) = &args.csv {
            let mut writer = WriterBuilder::new()
                .has_headers(false)
                .from_path(csv_path)
                .unwrap();

            let details: FrameDetails = parser.details.clone().into();

            frames.iter().enumerate().for_each(|(index, frame)| {
                // write headers
                if index == 0 {
                    writer.write_record(get_headers(frame)).unwrap();
                }
                // write frame with details
                writer
                    .serialize(FrameWithDetails {
                        frame: &frame,
                        details: &details,
                    })
                    .unwrap();
            })
        }
    }
}

fn get_headers(frame: &Frame) -> Vec<String> {
    let mut headers = vec![
        "CUSTOM.dateTime".to_string(),    // Custom date and time of the frame
        "OSD.flyTime".to_string(),        // Flight time in seconds
        "OSD.latitude".to_string(),       // Latitude in degrees
        "OSD.longitude".to_string(),      // Longitude in degrees
        "OSD.height".to_string(),         // Height above ground level in meters
        "OSD.heightMax".to_string(),      // Maximum height reached in meters
        "OSD.vpsHeight".to_string(),      // Visual Positioning System height in meters
        "OSD.altitude".to_string(),       // Altitude above sea level in meters
        "OSD.xSpeed".to_string(),         // Speed along the X-axis in meters per second
        "OSD.xSpeedMax".to_string(), // Maximum speed reached along the X-axis in meters per second
        "OSD.ySpeed".to_string(),    // Speed along the Y-axis in meters per second
        "OSD.ySpeedMax".to_string(), // Maximum speed reached along the Y-axis in meters per second
        "OSD.zSpeed".to_string(),    // Vertical speed in meters per second
        "OSD.zSpeedMax".to_string(), // Maximum vertical speed reached in meters per second
        "OSD.pitch".to_string(),     // Pitch angle in degrees
        "OSD.roll".to_string(),      // Roll angle in degrees
        "OSD.yaw".to_string(),       // Yaw angle in degrees
        "OSD.flycState".to_string(), // Current flight mode
        "OSD.flycCommand".to_string(), // Current app command
        "OSD.flightAction".to_string(), // Current flight action
        "OSD.isGPSUsed".to_string(), // Indicates if GPS is being used
        "OSD.nonGPSCause".to_string(), // Reason for not using GPS
        "OSD.gpsNum".to_string(),    // Number of GPS satellites detected
        "OSD.gpsLevel".to_string(),  // GPS signal level
        "OSD.droneType".to_string(), // Type of drone
        "OSD.isSwaveWork".to_string(), // Indicates if obstacle avoidance is active
        "OSD.waveError".to_string(), // Indicates if there's an error with obstacle avoidance
        "OSD.goHomeStatus".to_string(), // Current status of the return-to-home function
        "OSD.batteryType".to_string(), // Type of battery
        "OSD.isOnGround".to_string(), // Indicates if the drone is on the ground
        "OSD.isMotorOn".to_string(), // Indicates if the motor is running
        "OSD.isMotorBlocked".to_string(), // Indicates if the motor is blocked
        "OSD.motorStartFailedCause".to_string(), // Reason for motor start failure
        "OSD.isImuPreheated".to_string(), // Indicates if the IMU is preheated
        "OSD.imuInitFailReason".to_string(), // Reason for IMU initialization failure
        "OSD.isAcceleratorOverRange".to_string(), // Indicates if the accelerometer is over range
        "OSD.isBarometerDeadInAir".to_string(), // Indicates if the barometer is malfunctioning in air
        "OSD.isCompassError".to_string(),       // Indicates if there's a compass error
        "OSD.isGoHomeHeightModified".to_string(), // Indicates if the return-to-home height has been modified
        "OSD.canIOCWork".to_string(), // Indicates if Intelligent Orientation Control can work
        "OSD.isNotEnoughForce".to_string(), // Indicates if there's not enough force (e.g., low battery)
        "OSD.isOutOfLimit".to_string(),     // Indicates if the drone is out of its flight limit
        "OSD.isPropellerCatapult".to_string(), // Indicates if propeller catapult protection is active
        "OSD.isVibrating".to_string(),         // Indicates if the drone is experiencing vibrations
        "OSD.isVisionUsed".to_string(), // Indicates if vision positioning system is being used
        "OSD.voltageWarning".to_string(), // Battery voltage warning level
        "GIMBAL.mode".to_string(),      // Current gimbal mode
        "GIMBAL.pitch".to_string(),     // Gimbal pitch angle in degrees
        "GIMBAL.roll".to_string(),      // Gimbal roll angle in degrees
        "GIMBAL.yaw".to_string(),       // Gimbal yaw angle in degrees
        "GIMBAL.isPitchAtLimit".to_string(), // Indicates if gimbal pitch is at its limit
        "GIMBAL.isRollAtLimit".to_string(), // Indicates if gimbal roll is at its limit
        "GIMBAL.isYawAtLimit".to_string(), // Indicates if gimbal yaw is at its limit
        "GIMBAL.isStuck".to_string(),   // Indicates if the gimbal is stuck
        "CAMERA.isPhoto".to_string(),   // Indicates if the camera is in photo mode
        "CAMERA.isVideo".to_string(),   // Indicates if the camera is in video mode
        "CAMERA.sdCardIsInserted".to_string(), // Indicates if an SD card is inserted
        "CAMERA.sdCardState".to_string(), // Current state of the SD card
        "RC.downlinkSignal".to_string(), // Downlink signal strength
        "RC.uplinkSignal".to_string(),  // Uplink signal strength
        "RC.aileron".to_string(),       // Right stick horizontal position (aileron)
        "RC.elevator".to_string(),      // Right stick vertical position (elevator)
        "RC.throttle".to_string(),      // Left stick vertical position (throttle)
        "RC.rudder".to_string(),        // Left stick horizontal position (rudder)
        "BATTERY.chargeLevel".to_string(), // Battery charge level in percentage
        "BATTERY.voltage".to_string(),  // Battery voltage
        "BATTERY.current".to_string(),  // Battery current
        "BATTERY.currentCapacity".to_string(), // Current battery capacity
        "BATTERY.fullCapacity".to_string(), // Full battery capacity
        "BATTERY.cellNum".to_string(),  // Number of battery cells
        "BATTERY.isCellVoltageEstimated".to_string(), // Indicates if cell voltage is derived from global voltage
    ];

    // Cell voltages
    for i in 1..=frame.battery.cell_num {
        headers.push(format!("BATTERY.cellVoltage{}", i));
    }

    headers.extend(vec![
        "BATTERY.cellVoltageDeviation".to_string(), // Deviation in cell voltages
        "BATTERY.maxCellVoltageDeviation".to_string(), // Maximum deviation in cell voltages
        "BATTERY.temperature".to_string(),          // Battery temperature
        "BATTERY.minTemperature".to_string(),       // Minimum battery temperature
        "BATTERY.maxTemperature".to_string(),       // Maximum battery temperature
        "HOME.latitude".to_string(),                // Home point latitude in degrees
        "HOME.longitude".to_string(),               // Home point longitude in degrees
        "HOME.altitude".to_string(),                // Home point altitude in meters
        "HOME.heightLimit".to_string(),             // Max allowed height in meters
        "HOME.isHomeRecord".to_string(),            // Indicates if home point is recorded
        "HOME.goHomeMode".to_string(),              // Current return-to-home mode
        "HOME.isDynamicHomePointEnabled".to_string(), // Indicates if dynamic home point is enabled
        "HOME.isNearDistanceLimit".to_string(), // Indicates if the drone is near its distance limit
        "HOME.isNearHeightLimit".to_string(),   // Indicates if the drone is near its height limit
        "HOME.isCompassCalibrating".to_string(), // Indicates if compass calibration is in progress
        "HOME.compassCalibrationState".to_string(), // Current state of compass calibration
        "HOME.isMultipleModeEnabled".to_string(), // Indicates if multiple flight modes are enabled
        "HOME.isBeginnerMode".to_string(),      // Indicates if beginner mode is active
        "HOME.isIOCEnabled".to_string(), // Indicates if Intelligent Orientation Control is enabled
        "HOME.IOCMode".to_string(),      // Current Intelligent Orientation Control mode
        "HOME.goHomeHeight".to_string(), // Return-to-home height in meters
        "HOME.IOCCourseLockAngle".to_string(), // Intelligent Orientation Control course lock angle
        "HOME.maxAllowedHeight".to_string(), // Maximum allowed height for the drone in meters
        "HOME.currentFlightRecordIndex".to_string(), // Index of the current flight record
        "RECOVER.appPlatform".to_string(), // The platform of the app used (e.g., iOS, Android)
        "RECOVER.appVersion".to_string(), // Version of the app used
        "RECOVER.aircraftName".to_string(), // Name of the aircraft
        "RECOVER.aircraftSerial".to_string(), // Serial number of the aircraft
        "RECOVER.cameraSerial".to_string(), // Serial number of the camera
        "RECOVER.rcSerial".to_string(),  // Serial number of the remote control
        "RECOVER.batterySerial".to_string(), // Serial number of the battery
        "APP.tip".to_string(),           // App tip
        "APP.warn".to_string(),          // App warning
        "DETAILS.totalTime".to_string(), // Total flight time in seconds
        "DETAILS.totalDistance".to_string(), // Total distance flown in meters
        "DETAILS.maxHeight".to_string(), // Maximum height reached during the flight in meters
        "DETAILS.maxHorizontalSpeed".to_string(), // Maximum horizontal speed reached during the flight in meters per second
        "DETAILS.maxVerticalSpeed".to_string(), // Maximum vertical speed reached during the flight in meters per second
        "DETAILS.photoNum".to_string(),         // Number of photos taken during the flight
        "DETAILS.videoTime".to_string(),        // Total video recording time in seconds
        "DETAILS.aircraftName".to_string(),     // Name of the aircraft
        "DETAILS.aircraftSerial".to_string(),   // Serial number of the aircraft
        "DETAILS.cameraSerial".to_string(),     // Serial number of the camera
        "DETAILS.rcSerial".to_string(),         // Serial number of the remote control
        "DETAILS.appPlatform".to_string(),      // The platform of the app used (e.g., iOS, Android)
        "DETAILS.appVersion".to_string(),       // Version of the app used
    ]);

    headers
}
