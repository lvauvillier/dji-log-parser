use binrw::binread;
use chrono::{DateTime, Utc};
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::layout::details::{parse_battery_sn, Platform, ProductType};
use crate::utils::c_string;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { length: u16 })]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Recover {
    #[br(map = |x: u8| ProductType::from(x))]
    pub product_type: ProductType,
    #[br(map = |x: u8| Platform::from(x))]
    pub app_platform: Platform,
    #[br(map = |x: [u8; 3]| format!("{}.{}.{}", x[0], x[1], x[2]))]
    pub app_version: String,
    #[br(count = serial_length(length), map = c_string)]
    pub aircraft_sn: String,
    #[br(count = 32, map = c_string)]
    pub aircraft_name: String,
    #[br(map = |x: i64| DateTime::from_timestamp(x, 0).unwrap_or_default())]
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    pub timestamp: DateTime<Utc>,
    #[br(count = serial_length(length), map = c_string)]
    pub camera_sn: String,
    #[br(count = serial_length(length), map = c_string)]
    pub rc_sn: String,
    #[br(count = serial_length(length))]
    #[br(temp)]
    battery_buf: Vec<u8>,
    #[br(calc = parse_battery_sn(product_type, battery_buf))]
    pub battery_sn: String,
    /// Only in the records of 110 bytes
    #[br(if(length >= 110))]
    pub gimbal_type: Option<u8>,
}

/// Serial numbers are 10 bytes long in the records of 85 bytes, 16 bytes in the longer ones.
fn serial_length(length: u16) -> usize {
    if length < 109 {
        10
    } else {
        16
    }
}
