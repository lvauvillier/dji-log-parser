use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Custom {
    /// Date and time of the frame
    pub date_time: DateTime<Utc>,
}
