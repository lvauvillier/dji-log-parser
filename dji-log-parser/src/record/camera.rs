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
pub struct Camera {
    #[br(temp)]
    _bitpack1: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x01) == 1))]
    pub is_connect: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x02) == 1))]
    pub is_usb_connect: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0x04)))]
    pub timer_sync_state: u8,
    #[br(calc(sub_byte_field(_bitpack1, 0x38) == 1))]
    pub is_shooting_single_photo: bool,
    #[br(calc(sub_byte_field(_bitpack1, 0xC0) != 0))]
    pub is_recording: bool,

    #[br(temp)]
    _bitpack2: u8,
    #[br(calc(sub_byte_field(_bitpack2, 0x02) == 1))]
    pub has_sd_card: bool,
    #[br(calc(SDCardState::from(sub_byte_field(_bitpack2, 0x3C))))]
    pub sd_card_state: SDCardState,
    #[br(calc(sub_byte_field(_bitpack2, 0x40)))]
    pub is_upgrading: u8,

    #[br(temp)]
    _bitpack3: u8,
    #[br(calc(sub_byte_field(_bitpack3, 0x02) == 1))]
    pub is_heat: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x04) == 1))]
    pub is_capture_disable: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x08) == 1))]
    pub is_ddr_storing: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x10) == 1))]
    pub conti_capture: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0x20) == 1))]
    pub hdmi_output_status: bool,
    #[br(calc(sub_byte_field(_bitpack3, 0xC0)))]
    pub encrypt_status: u8,

    #[br(temp)]
    _bitpack4: u8,
    #[br(calc(sub_byte_field(_bitpack4, 0x01) == 1))]
    pub file_syn_state: bool,
    #[br(calc(sub_byte_field(_bitpack4, 0x02) == 1))]
    pub rc_btn_forbid_state: bool,
    #[br(calc(sub_byte_field(_bitpack4, 0x04) == 1))]
    pub get_focus_state: bool,
    #[br(calc(sub_byte_field(_bitpack4, 0x08) == 1))]
    pub pano_timelapse_gimbal_state: bool,
    #[br(calc(sub_byte_field(_bitpack4, 0x10) == 1))]
    pub is_enable_tracking_mode: bool,

    #[br(map = |x: u8| CameraWorkMode::from(x))]
    pub work_mode: CameraWorkMode,
    /// MB
    pub sd_card_total_capacity: u32,
    /// MB
    pub sd_card_remain_capacity: u32,
    pub remain_photo_num: u32,
    /// seconds
    pub remain_video_timer: u32,
    /// seconds
    pub record_time: u16,
    pub camera_type: u8,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum SDCardState {
    Normal,
    NoCard,
    InvalidCard,
    WriteProtected,
    Unformatted,
    Formatting,
    IllegalFileSys,
    Busy,
    Full,
    LowSpeed,
    IndexMax,
    Initialize,
    SuggestFormat,
    Repairing,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for SDCardState {
    fn from(value: u8) -> Self {
        match value {
            0 => SDCardState::Normal,
            1 => SDCardState::NoCard,
            2 => SDCardState::InvalidCard,
            3 => SDCardState::WriteProtected,
            4 => SDCardState::Unformatted,
            5 => SDCardState::Formatting,
            6 => SDCardState::IllegalFileSys,
            8 => SDCardState::Full,
            9 => SDCardState::LowSpeed,
            11 => SDCardState::IndexMax,
            12 => SDCardState::Initialize,
            13 => SDCardState::SuggestFormat,
            14 => SDCardState::Repairing,
            _ => SDCardState::Unknown(value),
        }
    }
}

#[derive(Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum CameraWorkMode {
    Capture,
    Recording,
    Playback,
    Transcode,
    Tuning,
    PowerSave,
    Download,
    XcodePlayback,
    Broadcast,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for CameraWorkMode {
    fn from(value: u8) -> Self {
        match value {
            0 => CameraWorkMode::Capture,
            1 => CameraWorkMode::Recording,
            2 => CameraWorkMode::Playback,
            3 => CameraWorkMode::Transcode,
            4 => CameraWorkMode::Tuning,
            5 => CameraWorkMode::PowerSave,
            6 => CameraWorkMode::Download,
            7 => CameraWorkMode::XcodePlayback,
            8 => CameraWorkMode::Broadcast,
            _ => CameraWorkMode::Unknown(value),
        }
    }
}
