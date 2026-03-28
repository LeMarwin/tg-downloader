//! Bot errors

pub mod ext;
pub use ext::*;

#[expect(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unrecognized URL: {0}")]
    UnrecognizedUrl(String),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to send: {0}")]
    RequestError(#[from] teloxide::RequestError),
    #[error("File too large: {0}, max is 20Mb")]
    FileTooLarge(u32),
    #[error("Failed to find resource id")]
    NoId,
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}
