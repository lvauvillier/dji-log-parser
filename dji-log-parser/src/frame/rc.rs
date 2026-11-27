use serde::Serialize;

use crate::record::mc_param::FailSafeProtectionType;
use crate::record::osd::DroneType;
use crate::record::rc_display_field::RCModeSwitch;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameRC {
    /// Downlink signal strength, in percent
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub downlink_signal: Option<u8>,
    /// Uplink signal strength, in percent
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub uplink_signal: Option<u8>,
    /// Right stick horizontal position (aileron)
    pub aileron: u16,
    /// Right stick vertical position (elevator)
    pub elevator: u16,
    /// Left stick vertical position (throttle)
    pub throttle: u16,
    /// Left stick horizontal position (rudder)
    pub rudder: u16,
    /// Position of the flight mode switch
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub mode_switch: Option<RCModeSwitch>,
    /// Right stick horizontal position (aileron), from -1 to 1
    pub aileron_normalized: f32,
    /// Right stick vertical position (elevator), from -1 to 1
    pub elevator_normalized: f32,
    /// Left stick vertical position (throttle), from -1 to 1
    pub throttle_normalized: f32,
    /// Left stick horizontal position (rudder), from -1 to 1
    pub rudder_normalized: f32,
    /// Action when the signal of the remote controller is lost
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub fail_safe_action: Option<FailSafeProtectionType>,
}

impl FrameRC {
    /// Sets the sticks, from 364 to 1684 with 1024 at the center.
    pub(crate) fn set_sticks(&mut self, aileron: u16, elevator: u16, throttle: u16, rudder: u16) {
        self.aileron = aileron;
        self.elevator = elevator;
        self.throttle = throttle;
        self.rudder = rudder;
        self.aileron_normalized = normalized_stick(aileron);
        self.elevator_normalized = normalized_stick(elevator);
        self.throttle_normalized = normalized_stick(throttle);
        self.rudder_normalized = normalized_stick(rudder);
    }
}

/// Stick position from -1 to 1, from its raw value: from 364 to 1684 with 1024 at the center, 0
/// when it isn't known.
pub(crate) fn normalized_stick(raw: u16) -> f32 {
    const CENTER: f32 = 1024.0;
    const RANGE: f32 = 660.0;
    if raw == 0 {
        0.0
    } else {
        (f32::from(raw) - CENTER) / RANGE
    }
}

/// Signal strength in percent, from the raw value of the OFDM records.
///
/// Wi-Fi and Lightbridge links give a percentage. OcuSync links give levels up to 90, such as
/// 0, 5, 15, 30, 55, 70 and 90, converted by linear steps.
pub(crate) fn signal_percent(raw: u8, drone_type: Option<DroneType>) -> u8 {
    if drone_type.is_some_and(has_percent_signal) {
        return raw;
    }

    // Raw values from and to, and percentages from and to
    const STEPS: [(u8, u8, f32, f32); 5] = [
        (7, 16, 1.0, 20.0),
        (17, 30, 21.0, 40.0),
        (31, 55, 41.0, 60.0),
        (56, 70, 61.0, 80.0),
        (71, 90, 81.0, 100.0),
    ];
    match STEPS.iter().find(|(_, to, _, _)| raw <= *to) {
        _ if raw < STEPS[0].0 => 0,
        Some(&(from, to, percent_from, percent_to)) => {
            let ratio = f32::from(raw - from) / f32::from(to - from);
            (percent_from + ratio * (percent_to - percent_from)).round() as u8
        }
        None => 100,
    }
}

/// Aircraft with a Wi-Fi or Lightbridge link
fn has_percent_signal(drone_type: DroneType) -> bool {
    use DroneType::*;
    matches!(
        drone_type,
        Inspire1
            | Phantom3Advanced
            | Phantom3Pro
            | Phantom3Standard
            | OpenFrame
            | AceOne
            | WKM
            | Naza
            | A2
            | A3
            | Phantom4
            | Matrice600
            | Phantom34K
            | Inspire2
            | Phantom4Pro
            | N3
            | Spark
            | Matrice600Pro
            | MavicAir
            | Matrice200
            | Phantom4Advanced
            | Matrice210
            | Phantom3SE
            | Matrice210RTK
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sticks_are_normalized_from_minus_1_to_1() {
        assert_eq!(normalized_stick(364), -1.0);
        assert_eq!(normalized_stick(1024), 0.0);
        assert_eq!(normalized_stick(1684), 1.0);
        assert_eq!(normalized_stick(0), 0.0);
    }

    #[test]
    fn ocusync_signal_levels() {
        let percents: Vec<u8> = [0, 5, 15, 30, 55, 70, 90]
            .into_iter()
            .map(|raw| signal_percent(raw, Some(DroneType::Mavic3Enterprise)))
            .collect();
        assert_eq!(percents, [0, 0, 18, 40, 60, 80, 100]);
        assert_eq!(signal_percent(127, None), 100);
    }

    #[test]
    fn lightbridge_signal_is_a_percentage() {
        assert_eq!(signal_percent(85, Some(DroneType::Phantom4Pro)), 85);
    }
}
