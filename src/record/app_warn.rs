use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { length: u16 })]
pub struct AppWarn {
    #[br(count=length, try_map = String::from_utf8)]
    pub warn: String,
}
