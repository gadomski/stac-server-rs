use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("backend error: {0}")]
    Backend(Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),
}
