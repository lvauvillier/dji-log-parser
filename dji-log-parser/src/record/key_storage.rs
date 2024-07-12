use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::keychain::FeaturePoint;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct KeyStorage {
    pub feature_point: FeaturePoint,
    #[br(temp)]
    data_length: u16,
    #[br(count = data_length)]
    pub data: Vec<u8>,
}
