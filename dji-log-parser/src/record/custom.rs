use binrw::binread;
use chrono::{DateTime, Datelike, Utc};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { version: u8 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Custom {
    #[br(temp)]
    _camera_shoot: u8,
    #[br(temp)]
    _video_shoot: u8,
    pub h_speed: f32,
    pub distance: f32,
    /// In seconds up to v2, then in milliseconds
    #[br(
        map = |x: i64| if version <= 2 {
            DateTime::from_timestamp(x, 0)
        } else {
            DateTime::from_timestamp(x / 1000, (x % 1000 * 1_000_000) as u32)
        }.unwrap_or_default(),
        // We ensure the year is between 2010 and 2100 to avoid invalid data
        assert(update_timestamp.year() > 2010 && update_timestamp.year() < 2100)
    )]
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    pub update_timestamp: DateTime<Utc>,
}
