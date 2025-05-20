use binrw::binread;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::io::SeekFrom;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[br(little, import(version: u8))]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct Details {
    #[br(count=20, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub sub_street: String,
    #[br(count=20, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub street: String,
    #[br(count=20, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub city: String,
    #[br(count=20, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub area: String,
    pub is_favorite: u8,
    pub is_new: u8,
    pub needs_upload: u8,
    pub record_line_count: i32,
    pub detail_info_checksum: i32,
    #[cfg_attr(target_arch = "wasm32", tsify(type = "string"))]
    #[br(map = |x: i64| DateTime::from_timestamp(x / 1000, (x % 1000 * 1000000) as u32).unwrap_or_default())]
    pub start_time: DateTime<Utc>,
    /// degrees
    pub longitude: f64,
    /// degrees
    pub latitude: f64,
    /// meters
    pub total_distance: f32,
    /// seconds
    #[br(map = |x: i32| x as f64 / 1000.0)]
    pub total_time: f64,
    /// meters
    pub max_height: f32,
    /// meters / seconds
    pub max_horizontal_speed: f32,
    /// meters / seconds
    pub max_vertical_speed: f32,
    pub capture_num: i32,
    pub video_time: i64,
    pub moment_pic_image_buffer_len: [i32; 4],
    pub moment_pic_shrink_image_buffer_len: [i32; 4],
    /// degrees
    #[br(map = |v: [f64; 4]| v.map(|rad: f64| rad.to_degrees()) )]
    pub moment_pic_longitude: [f64; 4],
    /// degrees
    #[br(map = |v: [f64; 4]| v.map(|rad: f64| rad.to_degrees()) )]
    pub moment_pic_latitude: [f64; 4],
    #[br(temp)]
    _analysis_offset: i64,
    #[br(temp)]
    _user_api_center_id_md5: [u8; 16],
    #[br(seek_before = if version <= 5 { SeekFrom::Start(352) } else { SeekFrom::Current(0) })]
    pub take_off_altitude: f32,
    #[br(
        seek_before = if version <= 5 { SeekFrom::Start(277) } else { SeekFrom::Current(0) },
        map = |x: u8| ProductType::from(x))
    ]
    pub product_type: ProductType,
    #[br(temp)]
    _activation_timestamp: i64,
    #[br(
        seek_before = if version <= 5 { SeekFrom::Start(278) } else { SeekFrom::Current(0) },
        count = if version <= 5 { 24 } else { 32 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string()
    )]
    pub aircraft_name: String,
    #[br(
        seek_before = if version <= 5 { SeekFrom::Start(267) } else { SeekFrom::Current(0) },
        count = if version <= 5 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string()
    )]
    pub aircraft_sn: String,
    #[br(
        seek_before = if version <= 5 { SeekFrom::Start(318) } else { SeekFrom::Current(0) },
        count = if version <= 5 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string()
    )]
    pub camera_sn: String,
    #[br(count = if version <= 5 { 10 } else { 16 }, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub rc_sn: String,
    #[br(count = if version <= 5 { 10 } else { 16 })]
    #[br(temp)]
    battery_buf: Vec<u8>,
    #[br(calc = parse_battery_sn(product_type, battery_buf))]
    pub battery_sn: String,
    #[br(map = |x: u8| Platform::from(x))]
    pub app_platform: Platform,
    #[br(map = |x: [u8; 3]| format!("{}.{}.{}", x[0], x[1], x[2]))]
    pub app_version: String,
}

#[derive(Serialize, Debug, Clone, PartialEq, Default, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum ProductType {
    #[default]
    None,
    Inspire1,
    Phantom3Standard,
    Phantom3Advanced,
    Phantom3Pro,
    OSMO,
    Matrice100,
    Phantom4,
    LB2,
    Inspire1Pro,
    A3,
    Matrice600,
    Phantom34K,
    MavicPro,
    ZenmuseXT,
    Inspire1RAW,
    A2,
    Inspire2,
    OSMOPro,
    OSMORaw,
    OSMOPlus,
    Mavic,
    OSMOMobile,
    OrangeCV600,
    Phantom4Pro,
    N3FC,
    Spark,
    Matrice600Pro,
    Phantom4Advanced,
    Phantom3SE,
    AG405,
    Matrice200,
    Matrice210,
    Matrice210RTK,
    MavicAir,
    Mavic2,
    Phantom4ProV2,
    Phantom4RTK,
    Phantom4Multispectral,
    Mavic2Enterprise,
    MavicMini,
    Matrice200V2,
    Matrice210V2,
    Matrice210RTKV2,
    MavicAir2,
    Matrice300RTK,
    FPV,
    MavicAir2S,
    Mini2,
    Mavic3,
    MiniSE,
    Mini3Pro,
    Mavic3Pro,
    Mini2SE,
    Matrice30,
    Mavic3Enterprise,
    Avata,
    Mini4Pro,
    Avata2,
    Matrice350RTK,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for ProductType {
    fn from(num: u8) -> Self {
        match num {
            0 => ProductType::None,
            1 => ProductType::Inspire1,
            2 => ProductType::Phantom3Standard,
            3 => ProductType::Phantom3Advanced,
            4 => ProductType::Phantom3Pro,
            5 => ProductType::OSMO,
            6 => ProductType::Matrice100,
            7 => ProductType::Phantom4,
            8 => ProductType::LB2,
            9 => ProductType::Inspire1Pro,
            10 => ProductType::A3,
            11 => ProductType::Matrice600,
            12 => ProductType::Phantom34K,
            13 => ProductType::MavicPro,
            14 => ProductType::ZenmuseXT,
            15 => ProductType::Inspire1RAW,
            16 => ProductType::A2,
            17 => ProductType::Inspire2,
            18 => ProductType::OSMOPro,
            19 => ProductType::OSMORaw,
            20 => ProductType::OSMOPlus,
            21 => ProductType::Mavic,
            22 => ProductType::OSMOMobile,
            23 => ProductType::OrangeCV600,
            24 => ProductType::Phantom4Pro,
            25 => ProductType::N3FC,
            26 => ProductType::Spark,
            27 => ProductType::Matrice600Pro,
            28 => ProductType::Phantom4Advanced,
            29 => ProductType::Phantom3SE,
            30 => ProductType::AG405,
            31 => ProductType::Matrice200,
            33 => ProductType::Matrice210,
            34 => ProductType::Matrice210RTK,
            38 => ProductType::MavicAir,
            42 => ProductType::Mavic2,
            44 => ProductType::Phantom4ProV2,
            46 => ProductType::Phantom4RTK,
            57 => ProductType::Phantom4Multispectral,
            58 => ProductType::Mavic2Enterprise,
            59 => ProductType::MavicMini,
            60 => ProductType::Matrice200V2,
            61 => ProductType::Matrice210V2,
            62 => ProductType::Matrice210RTKV2,
            67 => ProductType::MavicAir2,
            70 => ProductType::Matrice300RTK,
            73 => ProductType::FPV,
            75 => ProductType::MavicAir2S,
            76 => ProductType::Mini2,
            77 => ProductType::Mavic3,
            96 => ProductType::MiniSE,
            103 => ProductType::Mini3Pro,
            111 => ProductType::Mavic3Pro,
            113 => ProductType::Mini2SE,
            116 => ProductType::Matrice30,
            118 => ProductType::Mavic3Enterprise,
            121 => ProductType::Avata,
            126 => ProductType::Mini4Pro,
            152 => ProductType::Avata2,
            170 => ProductType::Matrice350RTK,
            _ => ProductType::Unknown(num),
        }
    }
}

impl ProductType {
    pub fn battery_cell_num(&self) -> u8 {
        match self {
            ProductType::Inspire1 => 6,
            ProductType::Phantom3Standard => 4,
            ProductType::Phantom3Advanced => 4,
            ProductType::Phantom3Pro => 4,
            ProductType::Matrice100 => 6,
            ProductType::Phantom4 => 4,
            ProductType::Inspire1Pro => 6,
            ProductType::Matrice600 => 6,
            ProductType::Phantom34K => 4,
            ProductType::MavicPro => 3,
            ProductType::Inspire1RAW => 6,
            ProductType::Inspire2 => 6,
            ProductType::Mavic => 3,
            ProductType::Phantom4Pro => 4,
            ProductType::Spark => 3,
            ProductType::Matrice600Pro => 6,
            ProductType::Phantom4Advanced => 4,
            ProductType::Phantom3SE => 4,
            ProductType::Matrice200 => 6,
            ProductType::Matrice210 => 6,
            ProductType::Matrice210RTK => 6,
            ProductType::MavicAir => 3,
            ProductType::Mavic2 => 4,
            ProductType::Phantom4ProV2 => 4,
            ProductType::Phantom4RTK => 4,
            ProductType::Phantom4Multispectral => 4,
            ProductType::Mavic2Enterprise => 4,
            ProductType::MavicMini => 2,
            ProductType::Matrice200V2 => 6,
            ProductType::Matrice210V2 => 6,
            ProductType::Matrice210RTKV2 => 6,
            ProductType::MavicAir2 => 3,
            ProductType::Matrice300RTK => 12,
            ProductType::FPV => 6,
            ProductType::MavicAir2S => 3,
            ProductType::Mini2 => 2,
            ProductType::Mavic3 => 4,
            ProductType::MiniSE => 2,
            ProductType::Mini3Pro => 2,
            ProductType::Mavic3Pro => 4,
            ProductType::Mini2SE => 2,
            ProductType::Matrice30 => 6,
            ProductType::Mavic3Enterprise => 4,
            ProductType::Avata => 5,
            ProductType::Mini4Pro => 2,
            ProductType::Avata2 => 4,
            ProductType::Matrice350RTK => 12,
            _ => 4,
        }
    }

    pub fn battery_num(&self) -> u8 {
        match self {
            ProductType::Inspire1 => 2,
            ProductType::Phantom3Standard => 1,
            ProductType::Phantom3Advanced => 1,
            ProductType::Phantom3Pro => 1,
            ProductType::Matrice100 => 2,
            ProductType::Phantom4 => 1,
            ProductType::Inspire1Pro => 2,
            ProductType::Matrice600 => 6,
            ProductType::Phantom34K => 1,
            ProductType::MavicPro => 1,
            ProductType::Inspire1RAW => 2,
            ProductType::Inspire2 => 2,
            ProductType::Mavic => 1,
            ProductType::Phantom4Pro => 1,
            ProductType::Spark => 1,
            ProductType::Matrice600Pro => 6,
            ProductType::Phantom4Advanced => 1,
            ProductType::Phantom3SE => 1,
            ProductType::Matrice200 => 2,
            ProductType::Matrice210 => 2,
            ProductType::Matrice210RTK => 2,
            ProductType::MavicAir => 1,
            ProductType::Mavic2 => 1,
            ProductType::Phantom4ProV2 => 1,
            ProductType::Phantom4RTK => 1,
            ProductType::Phantom4Multispectral => 1,
            ProductType::Mavic2Enterprise => 1,
            ProductType::MavicMini => 1,
            ProductType::Matrice200V2 => 2,
            ProductType::Matrice210V2 => 2,
            ProductType::Matrice210RTKV2 => 2,
            ProductType::MavicAir2 => 1,
            ProductType::Matrice300RTK => 2,
            ProductType::FPV => 1,
            ProductType::MavicAir2S => 1,
            ProductType::Mini2 => 1,
            ProductType::Mavic3 => 1,
            ProductType::MiniSE => 1,
            ProductType::Mini3Pro => 1,
            ProductType::Mavic3Pro => 1,
            ProductType::Mini2SE => 1,
            ProductType::Matrice30 => 2,
            ProductType::Mavic3Enterprise => 1,
            ProductType::Avata => 1,
            ProductType::Mini4Pro => 1,
            ProductType::Avata2 => 1,
            ProductType::Matrice350RTK => 2,
            _ => 1,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum Platform {
    IOS,
    Android,
    DJIFly,
    Windows,
    Mac,
    Linux,
    #[serde(untagged)]
    Unknown(u8),
}

impl From<u8> for Platform {
    fn from(num: u8) -> Self {
        match num {
            1 => Platform::IOS,
            2 => Platform::Android,
            6 => Platform::DJIFly,
            10 => Platform::Windows,
            11 => Platform::Mac,
            12 => Platform::Linux,
            _ => Platform::Unknown(num),
        }
    }
}

/// Decode the battery serial number from raw bytes, choosing the method based on model:
/// - For Inspire1/Pro/RAW, interpret the low nibble of each byte as a BCD digit,
///   reverse the sequence, and trim leading `'0'`.
/// - Otherwise, treat the bytes as a UTF-8, null-terminated string.
///
pub fn parse_battery_sn(product_type: ProductType, buf: Vec<u8>) -> String {
    const BCD_PRODUCTS: [ProductType; 3] = [
        ProductType::Inspire1,
        ProductType::Inspire1Pro,
        ProductType::Inspire1RAW,
    ];

    if BCD_PRODUCTS.contains(&product_type) {
        decode_reversed_bcd_battery_sn(buf)
    } else {
        String::from_utf8_lossy(&buf).trim_end_matches('\0').to_string()
    }
}

/// Decode a reversed-BCD battery serial:
/// 1. Take each byte’s low 4 bits (0–9) as a digit
/// 2. Reverse the digit sequence
/// 3. Skip any leading `'0'` characters
///
fn decode_reversed_bcd_battery_sn(buf: Vec<u8>) -> String {
    buf.into_iter()
        .map(|b| ((b & 0xF) + b'0') as char)
        .rev()
        .skip_while(|&c| c == '0')
        .collect()
}
