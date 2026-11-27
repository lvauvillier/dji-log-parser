use binrw::binread;
use serde::Serialize;
use std::f64::consts::PI;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;
use num_enum::FromPrimitive;

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
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub altitude: f32,
    /// meters / sec
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub speed_x: f32,
    /// meters / sec
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub speed_y: f32,
    /// meters / sec
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub speed_z: f32,
    /// degrees
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub pitch: f32,
    /// degrees
    #[br(map = |x: i16| x as f32 / 10.0)]
    pub roll: f32,
    /// degrees
    #[br(map = |x: i16| x as f32 / 10.0)]
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
    #[br(map = |x: u8| x as f32 / 10.0)]
    pub s_wave_height: f32,
    /// second
    #[br(map = |x: u16| x as f32 / 10.0)]
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

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum DroneType {
    None = 0,
    Inspire1 = 1,
    Phantom3Advanced = 2,
    Phantom3Pro = 3,
    Phantom3Standard = 4,
    OpenFrame = 5,
    AceOne = 6,
    WKM = 7,
    Naza = 8,
    A2 = 9,
    A3 = 10,
    Phantom4 = 11,
    Matrice600 = 14,
    Phantom34K = 15,
    MavicPro = 16,
    Inspire2 = 17,
    Phantom4Pro = 18,
    AgrasMG1 = 19,
    N3 = 20,
    Spark = 21,
    Matrice600Pro = 23,
    MavicAir = 24,
    Matrice200 = 25,
    Phantom4Advanced = 27,
    Matrice210 = 28,
    Phantom3SE = 29,
    Matrice210RTK = 30,
    Phantom4ProV2 = 36,
    AgrasMG1P = 38,
    Tello = 39,
    Phantom4RTK = 40,
    Mavic2 = 41,
    Matrice200V2 = 44,
    Matrice210V2 = 45,
    Matrice210RTKV2 = 46,
    Mavic2Enterprise = 51,
    MavicMini = 53,
    MavicAir2 = 58,
    Phantom4Multispectral = 59,
    Matrice300RTK = 60,
    FPV = 61,
    Mini2 = 63,
    AgrasT10 = 64,
    AgrasT30 = 65,
    MavicAir2S = 66,
    Matrice30 = 67,
    Mavic3 = 68,
    Mavic2EnterpriseAdvanced = 69,
    MiniSE = 70,
    AgrasT40 = 72,
    Mini3Pro = 73,
    Avata = 75,
    Inspire3 = 76,
    Mavic3Enterprise = 77,
    FlyCart30 = 78,
    Mavic3Pro = 84,
    Mavic3Classic = 86,
    Mini3 = 87,
    Mini2SE = 88,
    Matrice350RTK = 89,
    Air3 = 90,
    Matrice3TD = 91,
    Mini4Pro = 93,
    Avata2 = 94,
    Matrice4 = 99,
    Matrice4D = 100,
    Flip = 102,
    Matrice400 = 103,
    Neo = 104,
    Air3S = 105,
    Mini4K = 107,
    Mavic4Pro = 108,
    AgrasT100 = 109,
    Mini5Pro = 112,
    Neo2 = 124,
    Avata360 = 127,
    LitoX1 = 133,
    Lito1 = 140,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

// Not derived: num_enum reserves `#[default]` and it conflicts with `catch_all`
#[allow(clippy::derivable_impls)]
impl Default for DroneType {
    fn default() -> Self {
        DroneType::None
    }
}

#[derive(FromPrimitive, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum FlightMode {
    /// Manual mode. Shown as Manual in DJI app.
    Manual = 0,
    /// Attitude mode. Shown as Atti in DJI app.
    Atti = 1,
    /// Attitude course lock mode. Shown as Atti in DJI app.
    AttiCourseLock = 2,
    /// Attitude hover mode. Shown as Atti in DJI app.
    AttiHover = 3,
    /// Hover mode. Shown as P-GPS in DJI app.
    Hover = 4,
    /// GPS blake mode. Shown as P-GPS in DJI app.
    GPSBlake = 5,
    /// GPS Attitude mode. Shown as P-GPS in DJI app.
    GPSAtti = 6,
    /// GPS course lock mode. Shown as CL/P-CL/F-CL in DJI app.
    GPSCourseLock = 7,
    /// GPS Home mode. Shown as HL/P-HL/F-HL in DJI app.
    GPSHomeLock = 8,
    /// GPS hotpoint mode. Show as POI/F-POI in DJI app.
    GPSHotPoint = 9,
    /// Assisted takeoff mode. Shown as TakeOff in DJI app.
    AssistedTakeoff = 10,
    /// Auto takeoff mode. Shown as TakeOff in DJI app.
    AutoTakeoff = 11,
    /// Auto landing mode. Shown as Landing in DJI app.
    AutoLanding = 12,
    /// Attitude landing mode. Shown as Landing in DJI app.
    AttiLanding = 13,
    /// GPS waypoint mode. Shown as WP/F-WP in DJI app.
    GPSWaypoint = 14,
    /// Go home mode. Shown as Gohome in DJI app.
    GoHome = 15,
    ClickGo = 16,
    /// Joystick mode. hown as Joystick in DJI app.
    Joystick = 17,
    GPSAttiWristband = 18,
    Cinematic = 19,
    /// Attitude limited mode. Shown as Atti in DJI app.
    AttiLimited = 23,
    /// Draw mode. Shown as Draw in DJI app.
    Draw = 24,
    /// GPS follow me mode. Shown as FM/F-FM in DJI app.
    GPSFollowMe = 25,
    /// ActiveTrack mode. Shown as ActiveTrack in DJI app.
    ActiveTrack = 26,
    /// TapFly mode. Shown as TapFly in DJI app.
    TapFly = 27,
    Pano = 28,
    Farming = 29,
    FPV = 30,
    /// Sport mode. Shown as Sport in DJI app.
    GPSSport = 31,
    GPSNovice = 32,
    /// Confirm landing mode. Shown as Landing in DJI app.
    ConfirmLanding = 33,
    /// The aircraft should move following the terrain. Shown as TerrainTracking in DJI app.
    TerrainTracking = 35,
    NaviAdvGoHome = 36,
    NaviAdvLanding = 37,
    /// Tripod mode. Shown as Tripod in DJI app.
    Tripod = 38,
    /// Active track mode, corresponds to Spotlight active track mode. Shown as QuickShot/ActiveTrack in DJI app.
    TrackHeadlock = 39,
    /// The motors are just started. Shown as TakeOff in DJI app.
    EngineStart = 41,
    GPSGentle = 43,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum AppCommand {
    AutoFly = 1,
    AutoLanding = 2,
    HomePointNow = 3,
    HomePointHot = 4,
    HomePointLock = 5,
    GoHome = 6,
    StartMotor = 7,
    StopMotor = 8,
    Calibration = 9,
    DeformProtecClose = 10,
    DeformProtecOpen = 11,
    DropGoHome = 12,
    DropTakeOff = 13,
    DropLanding = 14,
    DynamicHomePointOpen = 15,
    DynamicHomePointClose = 16,
    FollowFunctionOpen = 17,
    FollowFunctionClose = 18,
    IOCOpen = 19,
    IOCClose = 20,
    DropCalibration = 21,
    PackMode = 22,
    UnPackMode = 23,
    EnterManualMode = 24,
    StopDeform = 25,
    DownDeform = 28,
    UpDeform = 29,
    ForceLanding = 30,
    ForceLanding2 = 31,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, PartialEq, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum GroundOrSky {
    #[num_enum(alternatives = [1])]
    Ground = 0,
    #[num_enum(alternatives = [3])]
    Sky = 2,
    // Explicit discriminant: the implicit one (3) would collide with Sky's alternative value
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8) = 4,
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum GoHomeStatus {
    Standby = 0,
    Preascending = 1,
    Align = 2,
    Ascending = 3,
    Cruise = 4,
    Braking = 5,
    Bypassing = 6,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum BatteryType {
    NonSmart = 1,
    Smart = 2,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum FlightAction {
    None = 0,
    WarningPowerGoHome = 1,
    WarningPowerLanding = 2,
    SmartPowerGoHome = 3,
    SmartPowerLanding = 4,
    LowVoltageLanding = 5,
    LowVoltageGoHome = 6,
    SeriousLowVoltageLanding = 7,
    RCOnekeyGoHome = 8,
    RCAssistantTakeoff = 9,
    RCAutoTakeoff = 10,
    RCAutoLanding = 11,
    AppAutoGoHome = 12,
    AppAutoLanding = 13,
    AppAutoTakeoff = 14,
    OutOfControlGoHome = 15,
    ApiAutoTakeoff = 16,
    ApiAutoLanding = 17,
    ApiAutoGoHome = 18,
    AvoidGroundLanding = 19,
    AirportAvoidLanding = 20,
    TooCloseGoHomeLanding = 21,
    TooFarGoHomeLanding = 22,
    AppWPMission = 23,
    WPAutoTakeoff = 24,
    GoHomeAvoid = 25,
    PGoHomeFinish = 26,
    VertLowLimitLanding = 27,
    BatteryForceLanding = 28,
    MCProtectGoHome = 29,
    MotorblockLanding = 30,
    AppRequestForceLanding = 31,
    FakeBatteryLanding = 32,
    RTHComingObstacleLanding = 33,
    IMUErrorRTH = 34,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum MotorStartFailedCause {
    None = 0,
    CompassError = 1,
    AssistantProtected = 2,
    DeviceLocked = 3,
    DistanceLimit = 4,
    IMUNeedCalibration = 5,
    IMUSNError = 6,
    IMUWarning = 7,
    CompassCalibrating = 8,
    AttiError = 9,
    NoviceProtected = 10,
    BatteryCellError = 11,
    BatteryCommuniteError = 12,
    SeriousLowVoltage = 13,
    SeriousLowPower = 14,
    LowVoltage = 15,
    TempureVolLow = 16,
    SmartLowToLand = 17,
    BatteryNotReady = 18,
    SimulatorMode = 19,
    PackMode = 20,
    AttitudeAbnormal = 21,
    UnActive = 22,
    FlyForbiddenError = 23,
    BiasError = 24,
    EscError = 25,
    ImuInitError = 26,
    SystemUpgrade = 27,
    SimulatorStarted = 28,
    ImuingError = 29,
    AttiAngleOver = 30,
    GyroscopeError = 31,
    AcceleratorError = 32,
    CompassFailed = 33,
    BarometerError = 34,
    BarometerNegative = 35,
    CompassBig = 36,
    GyroscopeBiasBig = 37,
    AcceleratorBiasBig = 38,
    CompassNoiseBig = 39,
    BarometerNoiseBig = 40,
    InvalidSn = 41,
    FlashOperating = 44,
    GPSdisconnect = 45,
    SDCardException = 47,
    IMUNoconnection = 61,
    RCCalibration = 62,
    RCCalibrationException = 63,
    RCCalibrationUnfinished = 64,
    RCCalibrationException2 = 65,
    RCCalibrationException3 = 66,
    AircraftTypeMismatch = 67,
    FoundUnfinishedModule = 68,
    CyroAbnormal = 70,
    BaroAbnormal = 71,
    CompassAbnormal = 72,
    GPSAbnormal = 73,
    NSAbnormal = 74,
    TopologyAbnormal = 75,
    RCNeedCali = 76,
    InvalidFloat = 77,
    M600BatTooLittle = 78,
    M600BatAuthErr = 79,
    M600BatCommErr = 80,
    M600BatDifVoltLarge1 = 81,
    M600BatDifVoltLarge2 = 82,
    InvalidVersion = 83,
    GimbalGyroAbnormal = 84,
    GimbalESCPitchNonData = 85,
    GimbalESCRollNonData = 86,
    GimbalESCYawNonData = 87,
    GimbalFirmwIsUpdating = 88,
    GimbalDisorder = 89,
    GimbalPitchShock = 90,
    GimbalRollShock = 91,
    GimbalYawShock = 92,
    IMUcCalibrationFinished = 93,
    BattVersionError = 101,
    RTKBadSignal = 102,
    RTKDeviationError = 103,
    ESCCalibrating = 112,
    GPSSignInvalid = 113,
    GimbalIsCalibrating = 114,
    LockByApp = 115,
    StartFlyHeightError = 116,
    ESCVersionNotMatch = 117,
    IMUOriNotMatch = 118,
    StopByApp = 119,
    #[num_enum(alternatives = [122])]
    CompassIMUOriNotMatch = 120,
    BatteryOverTemperature = 123,
    BatteryInstallError = 124,
    BeImpact = 125,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum NonGPSCause {
    Already = 0,
    Forbid = 1,
    GpsNumNonEnough = 2,
    GpsHdopLarge = 3,
    GpsPositionNonMatch = 4,
    SpeedErrorLarge = 5,
    YawErrorLarge = 6,
    CompassErrorLarge = 7,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum ImuInitFailReason {
    MonitorError = 0,
    CollectingData = 1,
    AcceDead = 3,
    CompassDead = 4,
    BarometerDead = 5,
    BarometerNegative = 6,
    CompassModTooLarge = 7,
    GyroBiasTooLarge = 8,
    AcceBiasTooLarge = 9,
    CompassNoiseTooLarge = 10,
    BarometerNoiseTooLarge = 11,
    WaitingMcStationary = 12,
    AcceMoveTooLarge = 13,
    McHeaderMoved = 14,
    McVibrated = 15,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

// Not derived: num_enum reserves `#[default]` and it conflicts with `catch_all`
#[allow(clippy::derivable_impls)]
impl Default for ImuInitFailReason {
    fn default() -> Self {
        ImuInitFailReason::MonitorError
    }
}
