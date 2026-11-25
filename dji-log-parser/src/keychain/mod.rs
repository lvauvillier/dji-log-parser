use base64::engine::general_purpose::STANDARD as Base64Standard;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Error, Result};
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

const AES_IV_LENGTH: usize = 16;
const AES_KEY_LENGTH: usize = 32;

fn decode_aes_param(
    value: &str,
    expected_length: usize,
    name: &str,
    feature_point: FeaturePoint,
) -> Result<Vec<u8>> {
    let bytes = Base64Standard.decode(value).map_err(|e| {
        Error::InvalidKeychain(format!("{feature_point:?} {name} is not valid base64: {e}"))
    })?;
    if bytes.len() != expected_length {
        return Err(Error::InvalidKeychain(format!(
            "{feature_point:?} {name} is {} bytes long, expected {expected_length}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

impl Keychain {
    pub fn empty() -> Self {
        Keychain(HashMap::new())
    }

    /// Builds a keychain from the feature points returned by the DJI API.
    ///
    /// Fails if an iv or key is not valid base64, or doesn't have the AES-256 CBC length.
    pub fn from_feature_points(keychain_entries: &[KeychainFeaturePoint]) -> Result<Self> {
        keychain_entries
            .iter()
            .map(|entry| {
                let iv = decode_aes_param(&entry.aes_iv, AES_IV_LENGTH, "iv", entry.feature_point)?;
                let key =
                    decode_aes_param(&entry.aes_key, AES_KEY_LENGTH, "key", entry.feature_point)?;
                Ok((entry.feature_point, (iv, key)))
            })
            .collect::<Result<_>>()
            .map(Keychain)
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
