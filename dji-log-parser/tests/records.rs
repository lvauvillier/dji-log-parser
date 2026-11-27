//! Record values checked against what is known of the recorded flights.

use std::path::PathBuf;

use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::record::rc::FlightModeSwitch;
use dji_log_parser::record::smart_battery_group::{SmartBatteryGroup, SmartBatteryStatic};
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("unable to read {}: {e}", path.display()))
}

fn keychains(name: &str) -> Vec<Vec<KeychainFeaturePoint>> {
    serde_json::from_slice(&fixture(name)).unwrap()
}

fn flight_mode_switches(name: &str) -> Vec<FlightModeSwitch> {
    let parser = DJILog::from_bytes(fixture(name)).unwrap();
    parser
        .records(None)
        .unwrap()
        .into_iter()
        .filter_map(|record| match record {
            Record::RC(rc) => Some(rc.flight_mode_switch),
            _ => None,
        })
        .collect()
}

/// These Mavic Pro flights were flown in P mode, the position Two of its flight mode switch.
#[test]
fn mavic_pro_p_mode_is_position_two() {
    for name in ["v8_mavicpro.txt", "v8_mavicpro_2017.txt"] {
        let switches = flight_mode_switches(name);
        assert!(!switches.is_empty(), "{name}: no RC record");
        assert!(
            switches
                .iter()
                .all(|switch| *switch == FlightModeSwitch::Two),
            "{name}: {switches:?}"
        );
    }
}

fn smart_battery_statics(name: &str, keychains: &str) -> Vec<SmartBatteryStatic> {
    let parser = DJILog::from_bytes(fixture(name)).unwrap();
    parser
        .records(Some(self::keychains(keychains)))
        .unwrap()
        .into_iter()
        .filter_map(|record| match record {
            Record::SmartBatteryGroup(SmartBatteryGroup::SmartBatteryStatic(battery)) => {
                Some(battery)
            }
            _ => None,
        })
        .collect()
}

/// The Mini 4 Pro battery has a capacity of 2590 mAh.
#[test]
fn mini_4_pro_battery_static_info() {
    let batteries = smart_battery_statics("v14_mini4pro.txt", "v14_mini4pro.keychains.json");
    assert!(!batteries.is_empty(), "no SmartBatteryStatic record");
    for battery in batteries {
        assert_eq!(battery.index, 0);
        assert_eq!(battery.designed_capacity, 2590);
    }
}

/// The 5 photos of this Mini 4 Pro flight keep their flag for several camera records, but each
/// photo is in one frame only.
#[test]
fn mini_4_pro_photos_are_in_one_frame_each() {
    let parser = DJILog::from_bytes(fixture("v14_mini4pro.txt")).unwrap();
    let keychains = Some(keychains("v14_mini4pro.keychains.json"));

    let photo_records = parser
        .records(keychains.clone())
        .unwrap()
        .into_iter()
        .filter(
            |record| matches!(record, Record::Camera(camera) if camera.is_shooting_single_photo),
        )
        .count();
    let photo_frames = parser
        .frames(keychains)
        .unwrap()
        .into_iter()
        .filter(|frame| frame.camera.is_photo)
        .count();

    assert!(
        photo_records > 5,
        "{photo_records} camera records with a photo"
    );
    assert_eq!(photo_frames, 5);
}
