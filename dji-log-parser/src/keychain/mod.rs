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

/// AES-256 CBC parameters used to decrypt the records of a `FeaturePoint`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AesParams {
    pub iv: [u8; 16],
    pub key: [u8; 32],
}

/// `Keychain` serves as more convenient access to decrypt `Record` instances.
/// It associates each `FeaturePoint` with its AES parameters. The iv is updated after each
/// decrypted record, as it is chained from one record to the next.
pub(crate) struct Keychain(HashMap<FeaturePoint, AesParams>);

fn decode_aes_param<const N: usize>(
    value: &str,
    name: &str,
    feature_point: FeaturePoint,
) -> Result<[u8; N]> {
    let bytes = Base64Standard.decode(value).map_err(|e| {
        Error::InvalidKeychain(format!("{feature_point:?} {name} is not valid base64: {e}"))
    })?;
    <[u8; N]>::try_from(bytes.as_slice()).map_err(|_| {
        Error::InvalidKeychain(format!(
            "{feature_point:?} {name} is {} bytes long, expected {N}",
            bytes.len()
        ))
    })
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
                let iv = decode_aes_param(&entry.aes_iv, "iv", entry.feature_point)?;
                let key = decode_aes_param(&entry.aes_key, "key", entry.feature_point)?;
                Ok((entry.feature_point, AesParams { iv, key }))
            })
            .collect::<Result<_>>()
            .map(Keychain)
    }

    pub fn get_mut(&mut self, feature_point: &FeaturePoint) -> Option<&mut AesParams> {
        self.0.get_mut(feature_point)
    }
}
