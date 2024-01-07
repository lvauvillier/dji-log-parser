use binrw::binread;
use serde::Serialize;

use crate::layout::feature_point::FeaturePoint;

#[binread]
#[derive(Serialize, Debug)]
#[br(little)]
pub struct KeyStorage {
    pub feature_point: FeaturePoint,
    #[br(temp)]
    data_length: u16,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}
