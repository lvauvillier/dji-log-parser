use dji_log_parser::keychain::{KeychainFeaturePoint, KeychainsResponse};
use dji_log_parser::DJILog;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_futures::js_sys::Promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Details")]
    pub type JSDetails;

    #[wasm_bindgen(typescript_type = "KeychainsRequest")]
    pub type JSKeychainsRequest;

    #[wasm_bindgen(typescript_type = "KeychainFeaturePoint[][]")]
    pub type JSKeychains;

    #[wasm_bindgen(typescript_type = "Record[]")]
    pub type JSRecords;

    #[wasm_bindgen(typescript_type = "Frame[]")]
    pub type JSFrames;

    #[wasm_bindgen(js_name = fetch)]
    fn js_fetch(input: &Request) -> Promise;
}

#[wasm_bindgen(js_name = DJILog)]
pub struct DJILogWrapper {
    inner: DJILog,
}

#[wasm_bindgen(js_class = DJILog)]
impl DJILogWrapper {
    /// Constructs a `DJILog` from an array of bytes.
    ///
    /// This function parses the Prefix and Info blocks of the log file,
    /// and handles different versions of the log format.
    ///
    /// # Arguments
    ///
    /// * `bytes` - An Uint8Array representing the DJI log file.
    ///
    #[wasm_bindgen(constructor)]
    pub fn from_bytes(bytes: &[u8]) -> Result<DJILogWrapper, JsValue> {
        DJILog::from_bytes(bytes.to_vec())
            .map(|value| DJILogWrapper { inner: value })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get version
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> u8 {
        self.inner.version
    }

    /// Get details
    #[wasm_bindgen(getter)]
    pub fn details(&self) -> Result<JSDetails, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.details)
            .map_err(|e| JsValue::from_str(&e.to_string()))
            .map(|value| value.unchecked_into())
    }

    /// Creates a `KeychainsRequest` object by parsing `KeyStorage` records.
    ///
    /// This function is used to build a request body for manually retrieving the keychain from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    /// For earlier versions, this function returns a default `KeychainsRequest`.
    ///
    #[wasm_bindgen(getter, js_name = "keychainsRequest")]
    pub fn keychains_request(&self) -> Result<JSKeychainsRequest, JsValue> {
        let keychain_request = self
            .inner
            .keychains_request()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&keychain_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))
            .map(|value| value.unchecked_into())
    }

    // Fetches keychains using the provided API key.
    ///
    /// This function first creates a `KeychainRequest` using the `keychain_request()` method,
    /// then uses that request to fetch the actual keychains from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A string slice that holds the API key for authentication with the DJI API.
    ///
    #[wasm_bindgen(js_name = "fetchKeychains")]
    pub async fn fetch_keychains(&self, api_key: &str) -> Result<JSKeychains, JsValue> {
        let keychain_request = self
            .inner
            .keychains_request()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let body = serde_json::to_string(&keychain_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let mut init = RequestInit::new();
        init.method("POST");
        init.body(Some(&JsValue::from_str(&body)));

        let request = Request::new_with_str_and_init(
            "https://dev.dji.com/openapi/v1/flight-records/keychains",
            &init,
        )?;

        request.headers().set("Content-Type", "application/json")?;
        request.headers().set("Api-Key", api_key)?;

        let response: Response = JsFuture::from(js_fetch(&request)).await?.unchecked_into();
        if !response.ok() {
            return Err(JsValue::from_str("Invalid Api Key"));
        }

        let json = JsFuture::from(response.json()?).await?;

        let keychain_response: KeychainsResponse = serde_wasm_bindgen::from_value(json)?;

        if keychain_response.result.code != 0 {
            return Err(JsValue::from_str(&format!(
                "DJI Api error: {}",
                keychain_response.result.msg
            )));
        }

        serde_wasm_bindgen::to_value(&keychain_response.data)
            .map_err(|e| JsValue::from_str(&e.to_string()))
            .map(|value| value.unchecked_into())
    }

    /// Retrieves the parsed raw records from the DJI log.
    ///
    /// This function decodes the raw records from the log file
    ///
    /// # Arguments
    ///
    /// * `keychains` - An optional vector of vectors containing `KeychainFeaturePoint` instances. This parameter
    ///   is used for decryption when working with encrypted logs (versions >= 13). If nothing is provided,
    ///   the function will attempt to process the log without decryption.
    ///
    #[wasm_bindgen]
    pub fn records(&self, keychains: Option<JSKeychains>) -> Result<JSRecords, JsValue> {
        let keychains: Option<Vec<Vec<KeychainFeaturePoint>>> = match keychains {
            Some(keychains) => {
                Some(serde_wasm_bindgen::from_value(keychains.unchecked_into()).unwrap())
            }
            None => None,
        };

        let records = self
            .inner
            .records(keychains)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&records)
            .map(|value| value.unchecked_into())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Retrieves the normalized frames from the DJI log.
    ///
    /// This function processes the raw records from the log file and converts them into standardized
    /// frames. Frames are a more user-friendly representation of the log data, normalized across all
    /// log versions for easier use and analysis.
    ///
    /// The function first decodes the raw records based on the specified decryption method, then
    /// converts these records into frames. This normalization process makes it easier to work with
    /// log data from different DJI log versions.
    ///
    /// # Arguments
    ///
    /// * `keychains` - An optional vector of vectors containing `KeychainFeaturePoint` instances. This parameter
    ///   is used for decryption when working with encrypted logs (versions >= 13). If nothing is provided,
    ///   the function will attempt to process the log without decryption.
    ///
    pub fn frames(&self, keychains: Option<JSKeychains>) -> Result<JSFrames, JsValue> {
        let keychains: Option<Vec<Vec<KeychainFeaturePoint>>> = match keychains {
            Some(keychains) => {
                Some(serde_wasm_bindgen::from_value(keychains.unchecked_into()).unwrap())
            }
            None => None,
        };

        let frames = self
            .inner
            .frames(keychains)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&frames)
            .map(|value| value.unchecked_into())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
