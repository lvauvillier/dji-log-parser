use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct VirtualStick {
    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(VirtualStickVerticalControlMode::from(sub_byte_field(_bitpack1, 0x30))))]
    pub vertical_control_mode: VirtualStickVerticalControlMode,
    #[br(calc(VirtualStickRollPitchControlMode::from(sub_byte_field(_bitpack1, 0xC0))))]
    pub roll_pitch_control_mode: VirtualStickRollPitchControlMode,
    #[br(calc(VirtualStickYawControlMode::from(sub_byte_field(_bitpack1, 0x08))))]
    pub yaw_control_mode: VirtualStickYawControlMode,
    #[br(calc(VirtualStickFlightCoordinateSystem::from(sub_byte_field(_bitpack1, 0x06))))]
    pub coordinate_system: VirtualStickFlightCoordinateSystem,

    /// Aircraft Roll. left and right panning [-30, 30] degrees
    pub roll: f32,
    /// Aircraft Pitch. forward or reverse [-30, 30] degrees.
    pub pitch: f32,
    /// Aircraft Yaw. left and right rotation [-180, 180] degrees
    pub yaw: f32,
    /// Aircraft throttle. Up or down [-5, 5]m/s
    pub throttle: f32,
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum VirtualStickVerticalControlMode {
    /// Sets the virtual stick vertical control values to be a vertical velocity.
    /// Positive and negative vertical velocity is for the aircraft ascending and
    /// descending respectively. Maximum vertical velocity is defined as 4 m/s. Minimum
    /// vertical velocity is defined as -4 m/s.
    Velocity,
    /// Sets the virtual stick vertical control values to be an altitude. Maximum
    /// position is defined as 500 m. Minimum position is defined as 0 m.
    Position,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for VirtualStickVerticalControlMode {
    fn from(value: u8) -> Self {
        match value {
            0 => VirtualStickVerticalControlMode::Velocity,
            1 => VirtualStickVerticalControlMode::Position,
            _ => VirtualStickVerticalControlMode::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum VirtualStickRollPitchControlMode {
    /// Sets the roll and pitch values to be an angle relative to a level aircraft. In
    /// the body coordinate system, positive and negative pitch angle is for the
    /// aircraft rotating about the y-axis in the positive direction or negative
    /// direction, respectively. Positive and negative roll angle is the positive
    /// direction or negative direction rotation angle about the x-axis, respectively.
    /// However in the ground coordinate system, positive and negative pitch angle is
    /// the angle value for the aircraft moving south and north, respectively. Positive
    /// and negative roll angle is the angle when the aircraft is moving east and west,
    /// respectively. Maximum angle is defined as 30 degrees. Minimum angle is defined
    /// as -30 degrees.
    Angle,
    /// Sets the roll and pitch values to be a velocity. In the body coordinate system,
    /// positive and negative pitch velocity is for the aircraft moving towards the
    /// positive direction or negative direction along the pitch axis and y-axis,
    /// respectively. Positive and negative roll velocity is when the aircraft is moving
    /// towards the positive direction or negative direction along the roll axis and
    /// x-axis, respectively. However, in the ground coordinate system, positive and
    /// negative pitch velocity is for the aircraft moving east and west, respectively.
    /// Positive and negative roll velocity is when the aircraft is moving north and
    /// south, respectively. Maximum velocity is defined as 15 meters/s. Minimum
    /// velocity is defined as -15 meters/s.
    Velocity,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for VirtualStickRollPitchControlMode {
    fn from(value: u8) -> Self {
        match value {
            0 => VirtualStickRollPitchControlMode::Angle,
            1 => VirtualStickRollPitchControlMode::Velocity,
            _ => VirtualStickRollPitchControlMode::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum VirtualStickYawControlMode {
    /// Sets the yaw values to be an angle relative to the north. Positive and negative
    /// yaw angle is for the aircraft rotating clockwise and counterclockwise,
    /// respectively. Maximum yaw angle is defined as 180 degrees. Minimum yaw angle is
    /// defined as -180 degrees.
    Angle,
    /// Sets the yaw values to be an angular velocity. Positive and negative angular
    /// velocity is for the aircraft rotating clockwise and counterclockwise,
    /// respectively. Maximum yaw angular velocity is defined as 100 degrees/s. Minimum
    /// yaw angular velocity is defined as -100 degrees/s.
    Velocity,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for VirtualStickYawControlMode {
    fn from(value: u8) -> Self {
        match value {
            0 => VirtualStickYawControlMode::Angle,
            1 => VirtualStickYawControlMode::Velocity,
            _ => VirtualStickYawControlMode::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum VirtualStickFlightCoordinateSystem {
    /// Ground coordinate system.
    Ground,
    /// Body coordinate system.
    Body,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for VirtualStickFlightCoordinateSystem {
    fn from(value: u8) -> Self {
        match value {
            0 => VirtualStickFlightCoordinateSystem::Ground,
            1 => VirtualStickFlightCoordinateSystem::Body,
            _ => VirtualStickFlightCoordinateSystem::Unknown(value),
        }
    }
}
