use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
pub enum Error {
    /// [std::io::Error]
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// [stac_api_backend::Error]
    #[error(transparent)]
    StacApiBackend(#[from] stac_api_backend::Error),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
