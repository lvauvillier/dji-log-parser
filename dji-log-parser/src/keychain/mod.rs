use std::collections::HashMap;

mod feature_point;
mod request;
mod response;

pub use feature_point::FeaturePoint;
pub use request::*;
pub use response::*;

/// `Keychain` serves as a mapping to decrypt `Record` instances.
/// It associates each `FeaturePoint` with its corresponding AES initialization vector (IV)
/// and encryption key. In this hashmap, each `FeaturePoint` is linked to a tuple containing
/// the AES IV and key as array of bytes.
pub type Keychain = HashMap<FeaturePoint, (Vec<u8>, Vec<u8>)>;
