use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct ComponentSerial {
    #[br(map = |x: u16| ComponentType::from(x as u8))]
    pub component_type: ComponentType,
    #[br(temp)]
    length: u8,
    #[br(count=length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).trim_end_matches('\0').to_string())]
    pub serial: String,
}

#[derive(Serialize, Debug, Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub enum ComponentType {
    Camera,
    Aircraft,
    RC,
    Battery,
    #[serde(untagged)]
    Unknown(u8),
}

impl ComponentType {
    pub fn from(value: u8) -> Self {
        match value {
            1 => ComponentType::Camera,
            2 => ComponentType::Aircraft,
            3 => ComponentType::RC,
            4 => ComponentType::Battery,
            _ => ComponentType::Unknown(value),
        }
    }
}
