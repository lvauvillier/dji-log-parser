use base64::engine::general_purpose::STANDARD as Base64Standard;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

mod api;
mod feature_point;

pub use api::*;
pub use feature_point::FeaturePoint;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct EncodedKeychainFeaturePoint {
    pub feature_point: FeaturePoint,
    pub aes_ciphertext: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct KeychainFeaturePoint {
    pub feature_point: FeaturePoint,
    pub aes_key: String,
    pub aes_iv: String,
}

/// `Keychain` serves as more convenient access to decrypt `Record` instances.
/// It associates each `FeaturePoint` with its corresponding AES initialization vector (IV)
/// and encryption key. In this hashmap, each `FeaturePoint` is linked to a tuple containing
/// the AES IV and key as array of bytes.
pub(crate) struct Keychain(HashMap<FeaturePoint, (Vec<u8>, Vec<u8>)>);

impl Keychain {
    pub fn empty() -> Self {
        Keychain(HashMap::new())
    }

    pub fn from_feature_points(keychain_entries: &Vec<KeychainFeaturePoint>) -> Self {
        Keychain(
            keychain_entries
                .into_iter()
                .map(|entry| {
                    (
                        entry.feature_point,
                        (
                            Base64Standard.decode(&entry.aes_iv).unwrap_or_default(),
                            Base64Standard.decode(&entry.aes_key).unwrap_or_default(),
                        ),
                    )
                })
                .collect(),
        )
    }

    pub fn get(&self, key: &FeaturePoint) -> Option<&(Vec<u8>, Vec<u8>)> {
        self.0.get(key)
    }

    pub fn insert(
        &mut self,
        key: FeaturePoint,
        value: (Vec<u8>, Vec<u8>),
    ) -> Option<(Vec<u8>, Vec<u8>)> {
        self.0.insert(key, value)
    }
}
