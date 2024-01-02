use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little, import { length: u16 })]
pub struct AppTip {
    #[br(count=length, try_map = |x| String::from_utf8(x))]
    pub tip: String,
}
