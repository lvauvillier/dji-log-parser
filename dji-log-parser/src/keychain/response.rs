use serde::Deserialize;

use super::feature_point::FeaturePoint;

/// Response structure received from the keychain API.
#[derive(Debug, Deserialize)]
pub struct KeychainResponse {
    pub data: Vec<Vec<KeychainAES>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeychainAES {
    pub feature_point: FeaturePoint,
    pub aes_key: String,
    pub aes_iv: String,
}
