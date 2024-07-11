use base64::engine::general_purpose::STANDARD as Base64Standard;
use base64::Engine as _;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

use super::feature_point::FeaturePoint;
use super::response::KeychainResponse;
use super::Keychain;

use crate::{Error, Result};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeychainCipherText {
    pub feature_point: FeaturePoint,
    pub aes_ciphertext: String,
}

/// Request structure for keychain API.
#[derive(Debug, Default, Serialize)]
pub struct KeychainRequest {
    pub version: u16,
    pub department: u8,
    #[serde(rename = "keychainsArray")]
    pub keychains: Vec<Vec<KeychainCipherText>>,
}

impl KeychainRequest {
    /// Fetches a `Vec<Keychain>` from the keychain API using the request details.
    /// Returns a result containing a vector of `Keychain`.
    pub fn fetch(&self, api_key: &str) -> Result<Vec<Keychain>> {
        let response = ureq::post("https://dev.dji.com/openapi/v1/flight-records/keychains")
            .set("Content-Type", "application/json")
            .set("Api-Key", api_key)
            .timeout(Duration::from_secs(30))
            .send_json(self)
            .map_err(|e| {
                if let ureq::Error::Status(403, _) = e {
                    Error::ApiKeyError
                } else {
                    Error::Network(e)
                }
            })?;

        let keychain_response: KeychainResponse = response.into_json()?;

        let result = keychain_response
            .data
            .iter()
            .map(|group| {
                let mut map = HashMap::new();
                for keychain_aes in group {
                    map.insert(
                        keychain_aes.feature_point,
                        (
                            Base64Standard
                                .decode(keychain_aes.aes_iv.clone())
                                .unwrap_or(Vec::new()),
                            Base64Standard
                                .decode(keychain_aes.aes_key.clone())
                                .unwrap_or(Vec::new()),
                        ),
                    );
                }
                map
            })
            .collect();

        Ok(result)
    }
}
