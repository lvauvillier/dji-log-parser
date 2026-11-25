//! Record values checked against what is known of the recorded flights.

use std::path::PathBuf;

use dji_log_parser::record::rc::FlightModeSwitch;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("unable to read {}: {e}", path.display()))
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
