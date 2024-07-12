use chrono::{DateTime, Utc};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct FrameCustom {
    /// Date and time of the frame
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    pub date_time: DateTime<Utc>,
}
