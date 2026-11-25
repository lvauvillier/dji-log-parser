use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::utils::sub_byte_field;
use num_enum::FromPrimitive;

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

#[derive(FromPrimitive, Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum SDCardState {
    Normal = 0,
    NoCard = 1,
    InvalidCard = 2,
    WriteProtected = 3,
    Unformatted = 4,
    Formatting = 5,
    IllegalFileSys = 6,
    Busy = 7,
    Full = 8,
    LowSpeed = 9,
    IndexMax = 11,
    Initialize = 12,
    SuggestFormat = 13,
    Repairing = 14,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}

#[derive(FromPrimitive, Serialize, Debug)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[repr(u8)]
pub enum CameraWorkMode {
    Capture = 0,
    Recording = 1,
    Playback = 2,
    Transcode = 3,
    Tuning = 4,
    PowerSave = 5,
    Download = 6,
    XcodePlayback = 7,
    Broadcast = 8,
    #[serde(untagged)]
    #[num_enum(catch_all)]
    Unknown(u8),
}
