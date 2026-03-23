pub mod ext;
pub use ext::*;

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
}
