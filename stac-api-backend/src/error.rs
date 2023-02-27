use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("backend error: {0}")]
    Backend(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    Stac(#[from] stac::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
