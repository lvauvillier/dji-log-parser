use serde::Serialize;

use crate::record::camera::SDCardState;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    /// Indicates if the camera is in photo mode
    pub is_photo: bool,
    /// Indicates if the camera is in video mode
    pub is_video: bool,
    /// Indicates if an SD card is inserted
    pub sd_card_is_inserted: bool,
    /// Current state of the SD card
    pub sd_card_state: Option<SDCardState>,
}
