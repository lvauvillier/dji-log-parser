use base64::engine::general_purpose::STANDARD as Base64Standard;
use base64::Engine as _;
#[cfg(target_arch = "wasm32")]
use ehttp::Mode;
use ehttp::{Headers, Request};
use serde::Serialize;
use std::collections::HashMap;

use super::feature_point::FeaturePoint;
use super::response::KeychainResponse;
use super::Keychain;

use crate::DJILogError;

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
    /// Asynchronously fetches `Vec<Keychain>` from the keychain API using the request details.
    /// The result is returned through a callback that takes a `Result<Vec<Keychain>, DJILogError>`.
    pub fn fetch<F>(&self, api_key: &str, callback: F)
    where
        F: FnOnce(Result<Vec<Keychain>, DJILogError>) + Send + 'static,
    {
        let headers = Headers::new(&[
            ("Accept", "*/*"),
            ("Content-Type", "application/json"),
            ("Api-Key", api_key),
        ]);

        let body = match serde_json::to_string(self)
            .map_err(|e| DJILogError::NetworkError(e.to_string()))
        {
            Ok(body) => body.into_bytes(),
            Err(e) => {
                callback(Err(e));
                return;
            }
        };

        let request = Request {
            method: "POST".to_owned(),
            url: "https://dev.dji.com/openapi/v1/flight-records/keychains".to_string(),
            body,
            headers,
            #[cfg(target_arch = "wasm32")]
            mode: Mode::default(),
        };

        ehttp::fetch(request, move |response| {
            let response = match response {
                Ok(response) => response,
                Err(e) => {
                    callback(Err(DJILogError::NetworkError(e.to_string())));
                    return;
                }
            };

            match serde_json::from_slice::<KeychainResponse>(&response.bytes)
                .map_err(|e| DJILogError::SerializeError(e.to_string()))
            {
                Ok(keychain_response) => {
                    let result: Vec<Keychain> = keychain_response
                        .data
                        .into_iter()
                        .map(|group| {
                            let mut map = HashMap::new();
                            for keychain_aes in group {
                                map.insert(
                                    keychain_aes.feature_point,
                                    (
                                        Base64Standard.decode(&keychain_aes.aes_iv).unwrap(),
                                        Base64Standard.decode(&keychain_aes.aes_key).unwrap(),
                                    ),
                                );
                            }
                            map
                        })
                        .collect();
                    callback(Ok(result));
                }
                Err(e) => callback(Err(e)),
            }
        });
    }
}
