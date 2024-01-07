use binrw::binread;
use serde::Serialize;

#[binread]
#[derive(Serialize, Debug)]
#[br(little, import { length: u16 })]
pub struct AppWarn {
    #[br(count=length, try_map = |x| String::from_utf8(x))]
    pub warn: String,
}
