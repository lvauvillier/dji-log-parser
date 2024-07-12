use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use super::{EncodedKeychainEntry, KeychainEntry};

#[cfg(not(target_arch = "wasm32"))]
use crate::{Error, Result};

/// Request structure for keychain API.
#[derive(Debug, Default, Serialize)]
pub struct KeychainsRequest {
    pub version: u16,
    pub department: u8,
    #[serde(rename = "keychainsArray")]
    pub keychains: Vec<Vec<EncodedKeychainEntry>>,
}

/// Response structure received from the keychain API.
#[derive(Debug, Deserialize)]
pub struct KeychainsResponse {
    pub data: Option<Vec<Vec<KeychainEntry>>>,
    pub result: KeychainResponseResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeychainResponseResult {
    pub code: u8,
    pub msg: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl KeychainsRequest {
    /// Fetches a `Vec<Keychain>` from the keychain API using the request details.
    /// Returns a result containing a vector of `Keychain`.
    pub fn fetch(&self, api_key: &str) -> Result<Vec<Vec<KeychainEntry>>> {
        let response = ureq::post("https://dev.dji.com/openapi/v1/flight-records/keychains")
            .set("Content-Type", "application/json")
            .set("Api-Key", api_key)
            .timeout(Duration::from_secs(30))
            .send_json(self)
            .map_err(|e| match e {
                ureq::Error::Status(403, _) => Error::ApiKeyError,
                ureq::Error::Status(status, _) => Error::NetworkRequestStatus(status),
                _ => Error::NetworkConnection,
            })?;

        let keychains_response: KeychainsResponse = response.into_json()?;

        if keychains_response.result.code != 0 {
            Err(Error::ApiError(keychains_response.result.msg))
        } else {
            match keychains_response.data {
                Some(data) => Ok(data),
                None => Err(Error::ApiError("Missing keychain data".to_owned())),
            }
        }
    }
}
