use thiserror::Error;

/// A crate-specific error type.
#[derive(Debug, Error)]
pub enum Error {
    /// An error originating from the backend.
    #[error("backend error: {0}")]
    Backend(Box<dyn std::error::Error + Send + Sync>),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
