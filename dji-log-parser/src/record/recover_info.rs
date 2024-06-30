use binrw::binread;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::layout::details::{Platform, ProductType};

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import {version: u8})]
pub struct RecoverInfo {
    #[br(map = |x: u8| ProductType::from(x))]
    pub product_type: ProductType,
    #[br(map = |x: u8| Platform::from(x))]
    pub app_platform: Platform,
    #[br(map = |x: [u8; 3]| format!("{}.{}.{}", x[0], x[1], x[2]))]
    pub app_version: String,
    #[br(count = if version <= 7 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub aircraft_sn: String,
    #[br(count = 32, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub aircraft_name: String,
    #[br(map = |x: i64| DateTime::from_timestamp(x, 0).unwrap_or_default())]
    pub timestamp: DateTime<Utc>,
    #[br(count = if version <= 7 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub camera_sn: String,
    #[br(count = if version <= 7 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub rc_sn: String,
    #[br(count = if version <= 7 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub battery_sn: String,
}
