use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

use crate::Result;

use super::{EncodedKeychainFeaturePoint, KeychainFeaturePoint};

const DEFAULT_ENDPOINT: &str = "https://dev.dji.com/openapi/v1/flight-records/keychains";

/// Request structure for keychain API.
#[derive(Debug, Default, Serialize, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct KeychainsRequest {
    pub version: u16,
    pub department: u8,
    #[serde(rename = "keychainsArray")]
    pub keychains: Vec<Vec<EncodedKeychainFeaturePoint>>,
}

/// Response structure received from the keychain API.
#[derive(Debug, Deserialize)]
pub struct KeychainsResponse {
    pub data: Option<Vec<Vec<KeychainFeaturePoint>>>,
    pub result: KeychainResponseResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeychainResponseResult {
    pub code: u8,
    pub msg: String,
}

impl KeychainsRequest {
    /// Sends a synchronous request to the keychain API.
    ///
    /// This method is only available for non-WASM targets.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication.
    /// * `endpoint` - The URL endpoint for the API.
    ///
    /// # Returns
    ///
    /// A Result containing a vector of vectors of KeychainFeaturePoint on success,
    /// or an Error on failure.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn fetch(
        &self,
        api_key: &str,
        endpoint: Option<&str>,
    ) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        let endpoint = endpoint.unwrap_or(DEFAULT_ENDPOINT);
        native::fetch(api_key, endpoint, self)
    }

    /// Sends an asynchronous request to the keychain API.
    ///
    /// This method is available for both WASM and non-WASM targets behind the `native-async` feature.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication.
    /// * `endpoint` - The URL endpoint for the API.
    ///
    /// # Returns
    ///
    /// A Future that resolves to a Result containing a vector of vectors of KeychainFeaturePoint on success,
    /// or an Error on failure.
    #[cfg(any(target_arch = "wasm32", feature = "native-async"))]
    pub async fn fetch_async(
        &self,
        api_key: &str,
        endpoint: Option<&str>,
    ) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        let endpoint = endpoint.unwrap_or(DEFAULT_ENDPOINT);

        #[cfg(not(target_arch = "wasm32"))]
        return native::fetch_async(api_key, endpoint, self).await;
        #[cfg(target_arch = "wasm32")]
        return wasm::fetch_async(api_key, endpoint, self).await;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod native {
    use std::time::Duration;

    use crate::keychain::KeychainFeaturePoint;
    use crate::{Error, Result};

    use super::{KeychainsRequest, KeychainsResponse};

    pub fn fetch(
        api_key: &str,
        endpoint: &str,
        request: &KeychainsRequest,
    ) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        let body = serde_json::to_string(&request)?;

        let response = ureq::post(endpoint)
            .set("Content-Type", "application/json")
            .set("Api-Key", api_key)
            .timeout(Duration::from_secs(30))
            .send_string(&body)
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

    #[cfg(feature = "native-async")]
    pub async fn fetch_async(
        api_key: &str,
        endpoint: &str,
        request: &KeychainsRequest,
    ) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        let api_key = api_key.to_string();
        let endpoint = endpoint.to_string();
        let request = request.clone();

        let (tx, rx) = async_channel::bounded(1);

        std::thread::spawn(move || {
            let response = fetch(&api_key, &endpoint, &request);
            tx.send(response);
        });

        rx.recv().await.map_err(|_| Error::NetworkConnection)?
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm {
    use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
    use wasm_bindgen_futures::js_sys::Promise;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    use crate::keychain::KeychainFeaturePoint;
    use crate::{Error, Result};

    use super::{KeychainsRequest, KeychainsResponse};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = fetch)]
        fn js_fetch(input: &Request) -> Promise;
    }

    pub async fn fetch_async(
        api_key: &str,
        endpoint: &str,
        request: &KeychainsRequest,
    ) -> Result<Vec<Vec<KeychainFeaturePoint>>> {
        let body = serde_json::to_string(&request)?;

        let mut init = RequestInit::new();
        init.method("POST");
        init.body(Some(&JsValue::from_str(&body)));

        let request = Request::new_with_str_and_init(endpoint, &init)
            .map_err(|_| Error::ApiError("Unable to init request".to_owned()))?;

        request
            .headers()
            .set("Content-Type", "application/json")
            .map_err(|_| Error::ApiError("Unable to set Content-Type header".to_owned()))?;
        request
            .headers()
            .set("Api-Key", &api_key)
            .map_err(|_| Error::ApiError("Unable to set Api-Key header".to_owned()))?;

        let response: Response = JsFuture::from(js_fetch(&request))
            .await
            .map_err(|_| Error::ApiError("Unable to fetch request".to_owned()))?
            .unchecked_into();

        if !response.ok() {
            return Err(Error::ApiKeyError);
        }

        let json = JsFuture::from(
            response
                .json()
                .map_err(|_| Error::ApiError("Unable get response".to_owned()))?,
        )
        .await
        .map_err(|_| Error::ApiError("Unable to get response".to_owned()))?;

        let keychains_response: KeychainsResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|_| Error::ApiError("Unable to parse response".to_owned()))?;

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
