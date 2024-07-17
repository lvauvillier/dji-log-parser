use base64::engine::general_purpose::STANDARD as Base64Standard;
use base64::Engine as _;
use serde::Serialize;
use std::collections::HashMap;
// use std::time::Duration;

use super::feature_point::FeaturePoint;
use super::response::KeychainResponse;
use super::Keychain;

use crate::DJILogError;

use log::{debug, error};


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
    pub fn fetch(&self, api_key: &str) -> Result<Vec<Keychain>, DJILogError> {
        debug!("Entering fetch method with API key: {}", api_key);
        let url = "https://dev.dji.com/openapi/v1/flight-records/keychains";
        
        let request_body = serde_json::json!({
            "version": self.version,
            "department": self.department,
            "keychainsArray": self.keychains
        });
    
        debug!("Request URL: {}", url);
        debug!("Request Body: {}", serde_json::to_string_pretty(&request_body).unwrap());
    
        let body = serde_json::to_string(&request_body)
            .map_err(|e| {
                error!("Failed to serialize request body: {:?}", e);
                DJILogError::SerializeError(e.to_string())
            })?;
    
        let response = ureq::post(url)
            .set("Content-Type", "application/json")
            .set("Api-Key", api_key)
            .timeout(std::time::Duration::from_secs(30))
            .send_string(&body)
            .map_err(|e| {
                error!("Failed to send request: {:?}", e);
                DJILogError::NetworkError(format!("Failed to send request: {}", e))
            })?;
    
        let status = response.status();
        debug!("Response Status: {}", status);
    
        let response_body = response.into_string()
            .map_err(|e| {
                error!("Failed to read response body: {:?}", e);
                DJILogError::NetworkError(format!("Failed to read response body: {}", e))
            })?;
    
        debug!("Response Body: {}", response_body);
    
        let response_json: serde_json::Value = serde_json::from_str(&response_body)
            .map_err(|e| {
                error!("Failed to parse response JSON: {:?}", e);
                DJILogError::DeserializeError(e.to_string())
            })?;
    
        // Check for API errors
        if let Some(result) = response_json.get("result") {
            if let Some(code) = result.get("code").and_then(|c| c.as_i64()) {
                if code != 0 {
                    let msg = result.get("msg").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                    error!("API error: {} - {}", code, msg);
                    return Err(DJILogError::NetworkError(format!("API error: {} - {}", code, msg)));
                }
            }
        }
    
        let keychain_response: KeychainResponse = if let Some(data) = response_json.get("data") {
            serde_json::from_value(data.clone())
                .map_err(|e| {
                    error!("Failed to deserialize keychain response: {:?}", e);
                    DJILogError::DeserializeError(e.to_string())
                })?
        } else {
            error!("Missing 'data' field in API response");
            return Err(DJILogError::DeserializeError("Missing 'data' field in API response".into()));
        };
    
        // Process the keychain response and return the result
        let result = keychain_response.data.iter()
            .map(|group| {
                group.iter().map(|keychain_aes| {
                    (
                        keychain_aes.feature_point.clone(),
                        (
                            Base64Standard.decode(&keychain_aes.aes_iv).unwrap(),
                            Base64Standard.decode(&keychain_aes.aes_key).unwrap(),
                        )
                    )
                }).collect::<HashMap<_, _>>()
            })
            .collect();
    
        debug!("Successfully fetched and processed keychains");
        Ok(result)
    }
}
