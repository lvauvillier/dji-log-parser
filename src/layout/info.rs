use binrw::binread;
use chrono::{DateTime, Utc};
use std::io::SeekFrom;

#[binread]
#[derive(Debug, Clone)]
#[br(little, import(format_version: u8))]
pub struct Info {
    #[br(count=20, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub sub_street: String,
    #[br(count=20, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub street: String,
    #[br(count=20, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub city: String,
    #[br(count=20, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub area: String,
    pub is_favorite: u8,
    pub is_new: u8,
    pub needs_upload: u8,
    pub record_line_count: i32,
    pub detail_info_checksum: i32,
    #[br(map = |x: i64| DateTime::from_timestamp(x / 1000, (x % 1000 * 1000000) as u32).unwrap())]
    pub timestamp: DateTime<Utc>,
    pub longitude: f64,
    pub latitude: f64,
    pub total_distance: f32,
    pub total_time: i32,
    pub max_height: f32,
    pub max_horizontal_speed: f32,
    pub max_vertical_speed: f32,
    pub capture_num: i32,
    pub video_time: i64,
    pub moment_pic_image_buffer_len: [i32; 4],
    pub moment_pic_shrink_image_buffer_len: [i32; 4],
    pub moment_pic_longitude: [f64; 4],
    pub moment_pic_latitude: [f64; 4],
    #[br(temp)]
    unknown: i64,
    #[br(temp, if(format_version > 5))]
    user_api_center_id_md5: [u8; 16],
    #[br(if(format_version > 5))]
    pub take_off_altitude: f32,
    #[br(if(format_version > 5))]
    pub product_type: u8,
    #[br(temp)]
    unknown2: i64,
    #[br(seek_before = if format_version <= 5 { SeekFrom::Start(278) } else { SeekFrom::Current(0)})]
    #[br(count=if format_version <= 5 { 24 } else {32}, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub aircraft_name: String,
    #[br(seek_before = if format_version <= 4 { SeekFrom::Start(267) } else { SeekFrom::Current(0)})]
    #[br(count=if format_version <= 5 { 10 } else {16}, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub aircraft_sn: String,
    #[br(seek_before = if format_version <= 4 { SeekFrom::Start(318) } else { SeekFrom::Current(0)})]
    #[br(count=if format_version <= 5 { 10 } else {16}, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub camera_sn: String,
    #[br(count=if format_version <= 5 { 10 } else {16}, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub rc_sn: String,
    #[br(count=if format_version <= 5 { 10 } else {16}, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    pub battery_sn: String,
    pub app_version: [u8; 4],
    #[br(temp)]
    unknown3: u8,
    #[br(temp)]
    reserved: [u8; 19],
    #[br(temp, if(format_version >= 12))]
    pub uuid: UUID,
}

#[binread]
#[derive(Debug, Clone)]
pub struct UUID([u8; 36]);

impl Default for UUID {
    fn default() -> Self {
        Self([0; 36])
    }
}
