use binrw::binread;
use serde::Serialize;
use std::f64::consts::PI;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br( import { version: u8 }, little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct OSD {
    /// degrees
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub longitude: f64,
    /// degrees
    #[br(map = |x: f64| (x * 180.0) / PI)]
    pub latitude: f64,
    /// meters
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub altitude: f32,
    /// meters / sec
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub speed_x: f32,
    /// meters / sec
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub speed_y: f32,
    /// meters / sec
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub speed_z: f32,
    /// degrees
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub pitch: f32,
    /// degrees
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub roll: f32,
    /// degrees
    #[br(map = |x: i16| (x as f32 / 10.0))]
    pub yaw: f32,

    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(FlightMode::from(sub_byte_field(_bitpack1, 0x7F))))]
    pub flight_mode: FlightMode,
    #[br(calc(sub_byte_field(_bitpack1, 0x80) == 1))]
    pub rc_outcontrol: bool,

    #[br(map = |x: u8| AppCommand::from(x))]
    pub app_command: AppCommand,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x01) == 1))]
    pub can_ioc_work: bool,
    #[br(calc(GroundOrSky::from(sub_byte_field(_bitpack2, 0x06))))]
    pub ground_or_sky: GroundOrSky,
    #[br(calc(sub_byte_field(_bitpack2, 0x08) == 1))]
    pub is_motor_up: bool,
    #[br(calc(sub_byte_field(_bitpack2, 0x10) == 1))]
    pub is_swave_work: bool,
    #[br(calc(GoHomeStatus::from(sub_byte_field(_bitpack2, 0xE0))))]
    pub go_home_status: GoHomeStatus,

    #[br(temp)]
    _bitpack3: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x01) == 1))]
    pub is_vision_used: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x06)))]
    pub voltage_warning: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x10) == 1))]
    pub is_imu_preheated: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x60)))]
    pub mode_channel: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x80) == 1))]
    pub is_gps_valid: bool,

    #[br(temp)]
    _bitpack4: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x01) == 1))]
    pub is_compass_error: bool,
    #[br(calc(sub_byte_field(_bitpack4, 0x02) == 1))]
    pub wave_error: bool,
    #[br(calc(sub_byte_field(_bitpack4, 0x3C)))]
    pub gps_level: u8,
    #[br(calc(BatteryType::from(sub_byte_field(_bitpack4, 0xC0))))]
    pub battery_type: BatteryType,

    #[br(temp)]
    _bitpack5: u8,
    #[br(calc(sub_byte_field(_bitpack5, 0x01) == 1))]
    pub is_out_of_limit: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x02) == 1))]
    pub is_go_home_height_modified: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x04) == 1))]
    pub is_propeller_catapult: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x08) == 1))]
    pub is_motor_blocked: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x10) == 1))]
    pub is_not_enough_force: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x20) == 1))]
    pub is_barometer_dead_in_air: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x40) == 1))]
    pub is_vibrating: bool,
    #[br(calc(sub_byte_field(_bitpack5, 0x80) == 1))]
    pub is_acceletor_over_range: bool,

    pub gps_num: u8,
    #[br(map = |x: u8| FlightAction::from(x))]
    pub flight_action: FlightAction,
    #[br(map = |x: u8| MotorStartFailedCause::from(x))]
    pub motor_start_failed_cause: MotorStartFailedCause,

    #[br(temp)]
    _bitpack6: u8,
    #[br(calc(NonGPSCause::from(sub_byte_field(_bitpack6, 0x0F))))]
    pub non_gps_cause: NonGPSCause,
    #[br(calc(sub_byte_field(_bitpack6, 0x10) == 1))]
    pub waypoint_limit_mode: bool,

    pub battery: u8,
    /// meters
    #[br(map = |x: u8| (x as f32 / 10.0))]
    pub s_wave_height: f32,
    /// second
    #[br(map = |x: u16| (x as f32 / 10.0))]
    pub fly_time: f32,
    pub motor_revolution: u8,
    #[br(temp)]
    _unknown: u16,
    pub version_c: u8,
    #[br(if(version >=2), map = |x: u8| DroneType::from(x))]
    pub drone_type: DroneType,
    #[br(if(version >=3), map = |x: u8| ImuInitFailReason::from(x))]
    pub imu_init_fail_reason: ImuInitFailReason,
}

#[derive(Serialize, Debug, Default, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum DroneType {
    #[default]
    None,
    Inspire1,
    Phantom3Advanced,
    Phantom3Pro,
    Phantom3Standard,
    OpenFrame,
    AceOne,
    WKM,
    Naza,
    A2,
    A3,
    Phantom4,
    Matrice600,
    Phantom34K,
    MavicPro,
    Inspire2,
    Phantom4Pro,
    N3,
    Spark,
    Matrice600Pro,
    MavicAir,
    Matrice200,
    Phantom4Advanced,
    Matrice210,
    Phantom3SE,
    Matrice210RTK,
    Phantom4ProV2,
    Mavic2,
    Mavic2Enterprise,
    MavicAir2,
    Matrice300RTK,
    Mini2,
    Mavic3Enterprise,
    Mavic3Pro,
    Matrice350RTK,
    Mini4Pro,
    Avata2,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for DroneType {
    fn from(value: u8) -> Self {
        match value {
            0 => DroneType::None,
            1 => DroneType::Inspire1,
            2 => DroneType::Phantom3Advanced,
            3 => DroneType::Phantom3Pro,
            4 => DroneType::Phantom3Standard,
            5 => DroneType::OpenFrame,
            6 => DroneType::AceOne,
            7 => DroneType::WKM,
            8 => DroneType::Naza,
            9 => DroneType::A2,
            10 => DroneType::A3,
            11 => DroneType::Phantom4,
            14 => DroneType::Matrice600,
            15 => DroneType::Phantom34K,
            16 => DroneType::MavicPro,
            17 => DroneType::Inspire2,
            18 => DroneType::Phantom4Pro,
            20 => DroneType::N3,
            21 => DroneType::Spark,
            23 => DroneType::Matrice600Pro,
            24 => DroneType::MavicAir,
            25 => DroneType::Matrice200,
            27 => DroneType::Phantom4Advanced,
            28 => DroneType::Matrice210,
            29 => DroneType::Phantom3SE,
            30 => DroneType::Matrice210RTK,
            36 => DroneType::Phantom4ProV2,
            41 => DroneType::Mavic2,
            51 => DroneType::Mavic2Enterprise,
            58 => DroneType::MavicAir2,
            60 => DroneType::Matrice300RTK,
            63 => DroneType::Mini2,
            77 => DroneType::Mavic3Enterprise,
            84 => DroneType::Mavic3Pro,
            89 => DroneType::Matrice350RTK,
            93 => DroneType::Mini4Pro,
            94 => DroneType::Avata2,
            _ => DroneType::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum FlightMode {
    /// Manual mode. Shown as Manual in DJI app.
    Manual,
    /// Attitude mode. Shown as Atti in DJI app.
    Atti,
    /// Attitude course lock mode. Shown as Atti in DJI app.
    AttiCourseLock,
    /// Attitude hover mode. Shown as Atti in DJI app.
    AttiHover,
    /// Hover mode. Shown as P-GPS in DJI app.
    Hover,
    /// GPS blake mode. Shown as P-GPS in DJI app.
    GPSBlake,
    /// GPS Attitude mode. Shown as P-GPS in DJI app.
    GPSAtti,
    /// GPS course lock mode. Shown as CL/P-CL/F-CL in DJI app.
    GPSCourseLock,
    /// GPS Home mode. Shown as HL/P-HL/F-HL in DJI app.
    GPSHomeLock,
    /// GPS hotpoint mode. Show as POI/F-POI in DJI app.
    GPSHotPoint,
    /// Assisted takeoff mode. Shown as TakeOff in DJI app.
    AssistedTakeoff,
    /// Auto takeoff mode. Shown as TakeOff in DJI app.
    AutoTakeoff,
    /// Auto landing mode. Shown as Landing in DJI app.
    AutoLanding,
    /// Attitude landing mode. Shown as Landing in DJI app.
    AttiLanding,
    /// GPS waypoint mode. Shown as WP/F-WP in DJI app.
    GPSWaypoint,
    /// Go home mode. Shown as Gohome in DJI app.
    GoHome,
    ClickGo,
    /// Joystick mode. hown as Joystick in DJI app.
    Joystick,
    GPSAttiWristband,
    Cinematic,
    /// Attitude limited mode. Shown as Atti in DJI app.
    AttiLimited,
    /// Draw mode. Shown as Draw in DJI app.
    Draw,
    /// GPS follow me mode. Shown as FM/F-FM in DJI app.
    GPSFollowMe,
    /// ActiveTrack mode. Shown as ActiveTrack in DJI app.
    ActiveTrack,
    /// TapFly mode. Shown as TapFly in DJI app.
    TapFly,
    Pano,
    Farming,
    FPV,
    /// Sport mode. Shown as Sport in DJI app.
    GPSSport,
    GPSNovice,
    /// Confirm landing mode. Shown as Landing in DJI app.
    ConfirmLanding,
    /// The aircraft should move following the terrain. Shown as TerrainTracking in DJI app.
    TerrainTracking,
    NaviAdvGoHome,
    NaviAdvLanding,
    /// Tripod mode. Shown as Tripod in DJI app.
    Tripod,
    /// Active track mode, corresponds to Spotlight active track mode. Shown as QuickShot/ActiveTrack in DJI app.
    TrackHeadlock,
    /// The motors are just started. Shown as TakeOff in DJI app.
    EngineStart,
    GPSGentle,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for FlightMode {
    fn from(value: u8) -> Self {
        match value {
            0 => FlightMode::Manual,
            1 => FlightMode::Atti,
            2 => FlightMode::AttiCourseLock,
            3 => FlightMode::AttiHover,
            4 => FlightMode::Hover,
            5 => FlightMode::GPSBlake,
            6 => FlightMode::GPSAtti,
            7 => FlightMode::GPSCourseLock,
            8 => FlightMode::GPSHomeLock,
            9 => FlightMode::GPSHotPoint,
            10 => FlightMode::AssistedTakeoff,
            11 => FlightMode::AutoTakeoff,
            12 => FlightMode::AutoLanding,
            13 => FlightMode::AttiLanding,
            14 => FlightMode::GPSWaypoint,
            15 => FlightMode::GoHome,
            16 => FlightMode::ClickGo,
            17 => FlightMode::Joystick,
            18 => FlightMode::GPSAttiWristband,
            19 => FlightMode::Cinematic,
            23 => FlightMode::AttiLimited,
            24 => FlightMode::Draw,
            25 => FlightMode::GPSFollowMe,
            26 => FlightMode::ActiveTrack,
            27 => FlightMode::TapFly,
            28 => FlightMode::Pano,
            29 => FlightMode::Farming,
            30 => FlightMode::FPV,
            31 => FlightMode::GPSSport,
            32 => FlightMode::GPSNovice,
            33 => FlightMode::ConfirmLanding,
            35 => FlightMode::TerrainTracking,
            36 => FlightMode::NaviAdvGoHome,
            37 => FlightMode::NaviAdvLanding,
            38 => FlightMode::Tripod,
            39 => FlightMode::TrackHeadlock,
            41 => FlightMode::EngineStart,
            43 => FlightMode::GPSGentle,
            _ => FlightMode::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum AppCommand {
    AutoFly,
    AutoLanding,
    HomePointNow,
    HomePointHot,
    HomePointLock,
    GoHome,
    StartMotor,
    StopMotor,
    Calibration,
    DeformProtecClose,
    DeformProtecOpen,
    DropGoHome,
    DropTakeOff,
    DropLanding,
    DynamicHomePointOpen,
    DynamicHomePointClose,
    FollowFunctionOpen,
    FollowFunctionClose,
    IOCOpen,
    IOCClose,
    DropCalibration,
    PackMode,
    UnPackMode,
    EnterManualMode,
    StopDeform,
    DownDeform,
    UpDeform,
    ForceLanding,
    ForceLanding2,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for AppCommand {
    fn from(value: u8) -> Self {
        match value {
            1 => AppCommand::AutoFly,
            2 => AppCommand::AutoLanding,
            3 => AppCommand::HomePointNow,
            4 => AppCommand::HomePointHot,
            5 => AppCommand::HomePointLock,
            6 => AppCommand::GoHome,
            7 => AppCommand::StartMotor,
            8 => AppCommand::StopMotor,
            9 => AppCommand::Calibration,
            10 => AppCommand::DeformProtecClose,
            11 => AppCommand::DeformProtecOpen,
            12 => AppCommand::DropGoHome,
            13 => AppCommand::DropTakeOff,
            14 => AppCommand::DropLanding,
            15 => AppCommand::DynamicHomePointOpen,
            16 => AppCommand::DynamicHomePointClose,
            17 => AppCommand::FollowFunctionOpen,
            18 => AppCommand::FollowFunctionClose,
            19 => AppCommand::IOCOpen,
            20 => AppCommand::IOCClose,
            21 => AppCommand::DropCalibration,
            22 => AppCommand::PackMode,
            23 => AppCommand::UnPackMode,
            24 => AppCommand::EnterManualMode,
            25 => AppCommand::StopDeform,
            28 => AppCommand::DownDeform,
            29 => AppCommand::UpDeform,
            30 => AppCommand::ForceLanding,
            31 => AppCommand::ForceLanding2,
            _ => AppCommand::Unknown(value),
        }
    }
}

#[derive(PartialEq, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum GroundOrSky {
    Ground,
    Sky,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for GroundOrSky {
    fn from(value: u8) -> Self {
        match value {
            0 | 1 => GroundOrSky::Ground,
            2 | 3 => GroundOrSky::Sky,
            _ => GroundOrSky::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum GoHomeStatus {
    Standby,
    Preascending,
    Align,
    Ascending,
    Cruise,
    Braking,
    Bypassing,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for GoHomeStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => GoHomeStatus::Standby,
            1 => GoHomeStatus::Preascending,
            2 => GoHomeStatus::Align,
            3 => GoHomeStatus::Ascending,
            4 => GoHomeStatus::Cruise,
            5 => GoHomeStatus::Braking,
            6 => GoHomeStatus::Bypassing,
            _ => GoHomeStatus::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum BatteryType {
    NonSmart,
    Smart,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for BatteryType {
    fn from(value: u8) -> Self {
        match value {
            1 => BatteryType::NonSmart,
            2 => BatteryType::Smart,
            _ => BatteryType::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum FlightAction {
    None,
    WarningPowerGoHome,
    WarningPowerLanding,
    SmartPowerGoHome,
    SmartPowerLanding,
    LowVoltageLanding,
    LowVoltageGoHome,
    SeriousLowVoltageLanding,
    RCOnekeyGoHome,
    RCAssistantTakeoff,
    RCAutoTakeoff,
    RCAutoLanding,
    AppAutoGoHome,
    AppAutoLanding,
    AppAutoTakeoff,
    OutOfControlGoHome,
    ApiAutoTakeoff,
    ApiAutoLanding,
    ApiAutoGoHome,
    AvoidGroundLanding,
    AirportAvoidLanding,
    TooCloseGoHomeLanding,
    TooFarGoHomeLanding,
    AppWPMission,
    WPAutoTakeoff,
    GoHomeAvoid,
    PGoHomeFinish,
    VertLowLimitLanding,
    BatteryForceLanding,
    MCProtectGoHome,
    MotorblockLanding,
    AppRequestForceLanding,
    FakeBatteryLanding,
    RTHComingObstacleLanding,
    IMUErrorRTH,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for FlightAction {
    fn from(value: u8) -> Self {
        match value {
            0 => FlightAction::None,
            1 => FlightAction::WarningPowerGoHome,
            2 => FlightAction::WarningPowerLanding,
            3 => FlightAction::SmartPowerGoHome,
            4 => FlightAction::SmartPowerLanding,
            5 => FlightAction::LowVoltageLanding,
            6 => FlightAction::LowVoltageGoHome,
            7 => FlightAction::SeriousLowVoltageLanding,
            8 => FlightAction::RCOnekeyGoHome,
            9 => FlightAction::RCAssistantTakeoff,
            10 => FlightAction::RCAutoTakeoff,
            11 => FlightAction::RCAutoLanding,
            12 => FlightAction::AppAutoGoHome,
            13 => FlightAction::AppAutoLanding,
            14 => FlightAction::AppAutoTakeoff,
            15 => FlightAction::OutOfControlGoHome,
            16 => FlightAction::ApiAutoTakeoff,
            17 => FlightAction::ApiAutoLanding,
            18 => FlightAction::ApiAutoGoHome,
            19 => FlightAction::AvoidGroundLanding,
            20 => FlightAction::AirportAvoidLanding,
            21 => FlightAction::TooCloseGoHomeLanding,
            22 => FlightAction::TooFarGoHomeLanding,
            23 => FlightAction::AppWPMission,
            24 => FlightAction::WPAutoTakeoff,
            25 => FlightAction::GoHomeAvoid,
            26 => FlightAction::PGoHomeFinish,
            27 => FlightAction::VertLowLimitLanding,
            28 => FlightAction::BatteryForceLanding,
            29 => FlightAction::MCProtectGoHome,
            30 => FlightAction::MotorblockLanding,
            31 => FlightAction::AppRequestForceLanding,
            32 => FlightAction::FakeBatteryLanding,
            33 => FlightAction::RTHComingObstacleLanding,
            34 => FlightAction::IMUErrorRTH,
            _ => FlightAction::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum MotorStartFailedCause {
    None,
    CompassError,
    AssistantProtected,
    DeviceLocked,
    DistanceLimit,
    IMUNeedCalibration,
    IMUSNError,
    IMUWarning,
    CompassCalibrating,
    AttiError,
    NoviceProtected,
    BatteryCellError,
    BatteryCommuniteError,
    SeriousLowVoltage,
    SeriousLowPower,
    LowVoltage,
    TempureVolLow,
    SmartLowToLand,
    BatteryNotReady,
    SimulatorMode,
    PackMode,
    AttitudeAbnormal,
    UnActive,
    FlyForbiddenError,
    BiasError,
    EscError,
    ImuInitError,
    SystemUpgrade,
    SimulatorStarted,
    ImuingError,
    AttiAngleOver,
    GyroscopeError,
    AcceleratorError,
    CompassFailed,
    BarometerError,
    BarometerNegative,
    CompassBig,
    GyroscopeBiasBig,
    AcceleratorBiasBig,
    CompassNoiseBig,
    BarometerNoiseBig,
    InvalidSn,
    FlashOperating,
    GPSdisconnect,
    SDCardException,
    IMUNoconnection,
    RCCalibration,
    RCCalibrationException,
    RCCalibrationUnfinished,
    RCCalibrationException2,
    RCCalibrationException3,
    AircraftTypeMismatch,
    FoundUnfinishedModule,
    CyroAbnormal,
    BaroAbnormal,
    CompassAbnormal,
    GPSAbnormal,
    NSAbnormal,
    TopologyAbnormal,
    RCNeedCali,
    InvalidFloat,
    M600BatTooLittle,
    M600BatAuthErr,
    M600BatCommErr,
    M600BatDifVoltLarge1,
    M600BatDifVoltLarge2,
    InvalidVersion,
    GimbalGyroAbnormal,
    GimbalESCPitchNonData,
    GimbalESCRollNonData,
    GimbalESCYawNonData,
    GimbalFirmwIsUpdating,
    GimbalDisorder,
    GimbalPitchShock,
    GimbalRollShock,
    GimbalYawShock,
    IMUcCalibrationFinished,
    BattVersionError,
    RTKBadSignal,
    RTKDeviationError,
    ESCCalibrating,
    GPSSignInvalid,
    GimbalIsCalibrating,
    LockByApp,
    StartFlyHeightError,
    ESCVersionNotMatch,
    IMUOriNotMatch,
    StopByApp,
    CompassIMUOriNotMatch,
    BatteryOverTemperature,
    BatteryInstallError,
    BeImpact,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for MotorStartFailedCause {
    fn from(value: u8) -> Self {
        match value {
            0 => MotorStartFailedCause::None,
            1 => MotorStartFailedCause::CompassError,
            2 => MotorStartFailedCause::AssistantProtected,
            3 => MotorStartFailedCause::DeviceLocked,
            4 => MotorStartFailedCause::DistanceLimit,
            5 => MotorStartFailedCause::IMUNeedCalibration,
            6 => MotorStartFailedCause::IMUSNError,
            7 => MotorStartFailedCause::IMUWarning,
            8 => MotorStartFailedCause::CompassCalibrating,
            9 => MotorStartFailedCause::AttiError,
            10 => MotorStartFailedCause::NoviceProtected,
            11 => MotorStartFailedCause::BatteryCellError,
            12 => MotorStartFailedCause::BatteryCommuniteError,
            13 => MotorStartFailedCause::SeriousLowVoltage,
            14 => MotorStartFailedCause::SeriousLowPower,
            15 => MotorStartFailedCause::LowVoltage,
            16 => MotorStartFailedCause::TempureVolLow,
            17 => MotorStartFailedCause::SmartLowToLand,
            18 => MotorStartFailedCause::BatteryNotReady,
            19 => MotorStartFailedCause::SimulatorMode,
            20 => MotorStartFailedCause::PackMode,
            21 => MotorStartFailedCause::AttitudeAbnormal,
            22 => MotorStartFailedCause::UnActive,
            23 => MotorStartFailedCause::FlyForbiddenError,
            24 => MotorStartFailedCause::BiasError,
            25 => MotorStartFailedCause::EscError,
            26 => MotorStartFailedCause::ImuInitError,
            27 => MotorStartFailedCause::SystemUpgrade,
            28 => MotorStartFailedCause::SimulatorStarted,
            29 => MotorStartFailedCause::ImuingError,
            30 => MotorStartFailedCause::AttiAngleOver,
            31 => MotorStartFailedCause::GyroscopeError,
            32 => MotorStartFailedCause::AcceleratorError,
            33 => MotorStartFailedCause::CompassFailed,
            34 => MotorStartFailedCause::BarometerError,
            35 => MotorStartFailedCause::BarometerNegative,
            36 => MotorStartFailedCause::CompassBig,
            37 => MotorStartFailedCause::GyroscopeBiasBig,
            38 => MotorStartFailedCause::AcceleratorBiasBig,
            39 => MotorStartFailedCause::CompassNoiseBig,
            40 => MotorStartFailedCause::BarometerNoiseBig,
            41 => MotorStartFailedCause::InvalidSn,
            44 => MotorStartFailedCause::FlashOperating,
            45 => MotorStartFailedCause::GPSdisconnect,
            47 => MotorStartFailedCause::SDCardException,
            61 => MotorStartFailedCause::IMUNoconnection,
            62 => MotorStartFailedCause::RCCalibration,
            63 => MotorStartFailedCause::RCCalibrationException,
            64 => MotorStartFailedCause::RCCalibrationUnfinished,
            65 => MotorStartFailedCause::RCCalibrationException2,
            66 => MotorStartFailedCause::RCCalibrationException3,
            67 => MotorStartFailedCause::AircraftTypeMismatch,
            68 => MotorStartFailedCause::FoundUnfinishedModule,
            70 => MotorStartFailedCause::CyroAbnormal,
            71 => MotorStartFailedCause::BaroAbnormal,
            72 => MotorStartFailedCause::CompassAbnormal,
            73 => MotorStartFailedCause::GPSAbnormal,
            74 => MotorStartFailedCause::NSAbnormal,
            75 => MotorStartFailedCause::TopologyAbnormal,
            76 => MotorStartFailedCause::RCNeedCali,
            77 => MotorStartFailedCause::InvalidFloat,
            78 => MotorStartFailedCause::M600BatTooLittle,
            79 => MotorStartFailedCause::M600BatAuthErr,
            80 => MotorStartFailedCause::M600BatCommErr,
            81 => MotorStartFailedCause::M600BatDifVoltLarge1,
            82 => MotorStartFailedCause::M600BatDifVoltLarge2,
            83 => MotorStartFailedCause::InvalidVersion,
            84 => MotorStartFailedCause::GimbalGyroAbnormal,
            85 => MotorStartFailedCause::GimbalESCPitchNonData,
            86 => MotorStartFailedCause::GimbalESCRollNonData,
            87 => MotorStartFailedCause::GimbalESCYawNonData,
            88 => MotorStartFailedCause::GimbalFirmwIsUpdating,
            89 => MotorStartFailedCause::GimbalDisorder,
            90 => MotorStartFailedCause::GimbalPitchShock,
            91 => MotorStartFailedCause::GimbalRollShock,
            92 => MotorStartFailedCause::GimbalYawShock,
            93 => MotorStartFailedCause::IMUcCalibrationFinished,
            101 => MotorStartFailedCause::BattVersionError,
            102 => MotorStartFailedCause::RTKBadSignal,
            103 => MotorStartFailedCause::RTKDeviationError,
            112 => MotorStartFailedCause::ESCCalibrating,
            113 => MotorStartFailedCause::GPSSignInvalid,
            114 => MotorStartFailedCause::GimbalIsCalibrating,
            115 => MotorStartFailedCause::LockByApp,
            116 => MotorStartFailedCause::StartFlyHeightError,
            117 => MotorStartFailedCause::ESCVersionNotMatch,
            118 => MotorStartFailedCause::IMUOriNotMatch,
            119 => MotorStartFailedCause::StopByApp,
            120 => MotorStartFailedCause::CompassIMUOriNotMatch,
            122 => MotorStartFailedCause::CompassIMUOriNotMatch,
            123 => MotorStartFailedCause::BatteryOverTemperature,
            124 => MotorStartFailedCause::BatteryInstallError,
            125 => MotorStartFailedCause::BeImpact,
            _ => MotorStartFailedCause::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum NonGPSCause {
    Already,
    Forbid,
    GpsNumNonEnough,
    GpsHdopLarge,
    GpsPositionNonMatch,
    SpeedErrorLarge,
    YawErrorLarge,
    CompassErrorLarge,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for NonGPSCause {
    fn from(value: u8) -> Self {
        match value {
            0 => NonGPSCause::Already,
            1 => NonGPSCause::Forbid,
            2 => NonGPSCause::GpsNumNonEnough,
            3 => NonGPSCause::GpsHdopLarge,
            4 => NonGPSCause::GpsPositionNonMatch,
            5 => NonGPSCause::SpeedErrorLarge,
            6 => NonGPSCause::YawErrorLarge,
            7 => NonGPSCause::CompassErrorLarge,
            _ => NonGPSCause::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug, Default, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum ImuInitFailReason {
    #[default]
    MonitorError,
    CollectingData,
    AcceDead,
    CompassDead,
    BarometerDead,
    BarometerNegative,
    CompassModTooLarge,
    GyroBiasTooLarge,
    AcceBiasTooLarge,
    CompassNoiseTooLarge,
    BarometerNoiseTooLarge,
    WaitingMcStationary,
    AcceMoveTooLarge,
    McHeaderMoved,
    McVibrated,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for ImuInitFailReason {
    fn from(value: u8) -> Self {
        match value {
            0 => ImuInitFailReason::MonitorError,
            1 => ImuInitFailReason::CollectingData,
            3 => ImuInitFailReason::AcceDead,
            4 => ImuInitFailReason::CompassDead,
            5 => ImuInitFailReason::BarometerDead,
            6 => ImuInitFailReason::BarometerNegative,
            7 => ImuInitFailReason::CompassModTooLarge,
            8 => ImuInitFailReason::GyroBiasTooLarge,
            9 => ImuInitFailReason::AcceBiasTooLarge,
            10 => ImuInitFailReason::CompassNoiseTooLarge,
            11 => ImuInitFailReason::BarometerNoiseTooLarge,
            12 => ImuInitFailReason::WaitingMcStationary,
            13 => ImuInitFailReason::AcceMoveTooLarge,
            14 => ImuInitFailReason::McHeaderMoved,
            15 => ImuInitFailReason::McVibrated,
            _ => ImuInitFailReason::Unknown(value),
        }
    }
}
