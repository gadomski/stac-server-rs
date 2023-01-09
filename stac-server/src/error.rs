use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
pub enum Error {
    /// stac_api::Error
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// url::ParseError
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
