use binrw::binread;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::layout::info::ProductType;

#[binread]
#[derive(Serialize, Debug)]
#[br(little, import {version: u8})]
pub struct RecoverInfo {
    #[br(map = |x: u8| ProductType::from(x))]
    pub product_type: ProductType,
    pub app_version: [u8; 4],
    #[br(count = if version <= 7 { 10 } else { 16 }, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub aircraft_sn: String,
    #[br(count = 32, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub aircraft_name: String,
    #[br(map = |x: i64| DateTime::from_timestamp(x, 0).unwrap_or_default())]
    pub timestamp: DateTime<Utc>,
    #[br(count = if version <= 7 { 10 } else { 16 }, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub camera_sn: String,
    #[br(count = if version <= 7 { 10 } else { 16 }, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub rc_sn: String,
    #[br(count = if version <= 7 { 10 } else { 16 }, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub battery_sn: String,
}
