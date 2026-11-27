use serde::Serialize;

use crate::record::rc_display_field::RCModeSwitch;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameRC {
    /// Downlink signal strength
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub downlink_signal: Option<u8>,
    /// Uplink signal strength
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
}
