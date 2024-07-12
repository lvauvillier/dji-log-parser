use binrw::binread;
use chrono::{DateTime, Utc};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Custom {
    #[br(temp)]
    _camera_shoot: u8,
    #[br(temp)]
    _video_shoot: u8,
    pub h_speed: f32,
    pub distance: f32,
    #[br(map = |x: i64| DateTime::from_timestamp(x / 1000, (x % 1000 * 1000000) as u32).unwrap_or_default())]
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    pub update_time_stamp: DateTime<Utc>,
}
