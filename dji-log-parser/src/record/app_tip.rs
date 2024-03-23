use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little, import { length: u16 })]
pub struct AppTip {
    #[br(count=length, try_map = String::from_utf8)]
    pub tip: String,
}
