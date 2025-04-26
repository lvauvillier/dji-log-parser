use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::layout::auxiliary::Department;
use dji_log_parser::DJILog;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};

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
    pub fn from_bytes(bytes: Vec<u8>) -> Result<DJILogWrapper, JsValue> {
        DJILog::from_bytes(bytes)
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
    #[wasm_bindgen(js_name = "keychainsRequest")]
    pub fn keychains_request(&self) -> Result<JSKeychainsRequest, JsValue> {
        let keychain_request = self
            .inner
            .keychains_request()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&keychain_request)
            .map_err(|e| JsValue::from_str(&e.to_string()))
            .map(|value| value.unchecked_into())
    }
    /// Creates a `KeychainsRequest` object by parsing `KeyStorage` records with manually specified params.
    ///
    /// This function is used to build a request body for manually retrieving the keychain from the DJI API.
    /// Keychains are required to decode records for logs with a version greater than or equal to 13.
    /// For earlier versions, this function returns a default `KeychainsRequest`.
    ///
    /// # Arguments
    ///
    /// * `department` - An optional `Department` to manually set in the request. If not provided, the department
    ///   will be determined from the log file.
    /// * `version` - An optional version number to manually set in the request. If not provided, the version
    ///   will be determined from the log file.
    ///
    #[wasm_bindgen(js_name = "keychainsRequestWithCustomParams")]
    pub fn keychains_request_with_custom_params(
        &self,
        department: Option<u8>,
        version: Option<u16>,
    ) -> Result<JSKeychainsRequest, JsValue> {
        let department = department.map(Department::from);

        let keychain_request = self
            .inner
            .keychains_request_with_custom_params(department, version)
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
    /// * `api_key` - A string that holds the API key for authentication with the DJI API.
    /// * `endpoint` - An optional string that specifies the endpoint for the DJI API. If not provided, a default endpoint will be used.
    ///
    #[wasm_bindgen(js_name = "fetchKeychains")]
    pub async fn fetch_keychains(
        &self,
        api_key: String,
        endpoint: Option<String>,
    ) -> Result<JSKeychains, JsValue> {
        let keychains = self
            .inner
            .keychains_request()
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .fetch_async(&api_key, endpoint.as_deref())
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&keychains)
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
