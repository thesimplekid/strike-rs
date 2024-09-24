use thiserror::Error;

/// Strike rs error
#[derive(Debug, Error)]
pub enum Error {
    /// Not Found
    #[error("Not found")]
    NotFound,
    /// Invalid Url
    #[error("Invalid Url")]
    InvalidUrl,
    /// From reqwest error
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    /// From reqwest error
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}
