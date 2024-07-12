use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid Api Key")]
    ApiKeyError,

    #[error("DJI Api error: {0}")]
    ApiError(String),

    #[error("Keychain is required")]
    KeychainRequired,

    #[error("Missing Auxilliary data: {0}")]
    MissingAuxilliaryData(String),

    #[error("Parse error")]
    Parse(#[from] binrw::Error),

    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error("Io error")]
    Io(#[from] std::io::Error),

    #[error("Base64 decode error")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Request request status error: {0}")]
    NetworkRequestStatus(u16),

    #[error("Network connection error")]
    NetworkConnection,
}
