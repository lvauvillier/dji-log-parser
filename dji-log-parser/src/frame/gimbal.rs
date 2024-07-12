use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::record::gimbal::GimbalMode;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameGimbal {
    /// Current gimbal mode
    #[cfg_attr(target_arch = "wasm32", tsify(optional))]
    pub mode: Option<GimbalMode>,
    /// Gimbal pitch angle in degrees
    pub pitch: f32,
    /// Gimbal roll angle in degrees
    pub roll: f32,
    /// Gimbal yaw angle in degrees
    pub yaw: f32,
    /// Indicates if gimbal pitch is at its limit
    pub is_pitch_at_limit: bool,
    /// Indicates if gimbal roll is at its limit
    pub is_roll_at_limit: bool,
    /// Indicates if gimbal yaw is at its limit
    pub is_yaw_at_limit: bool,
    /// Indicates if the gimbal is stuck
    pub is_stuck: bool,
}
