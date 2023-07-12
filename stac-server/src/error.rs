use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
pub enum Error {
    /// [std::net::AddrParseError]
    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),

    /// [hyper::Error]
    #[error(transparent)]
    Hyper(#[from] hyper::Error),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [serde_qs::Error]
    #[error(transparent)]
    SerdeQs(#[from] serde_qs::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// [stac_api_backend::Error]
    #[error(transparent)]
    StacApiBackend(#[from] stac_api_backend::Error),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
