use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("bb8 user error: {0}")]
    Bb8UserError(Box<dyn std::error::Error>),

    #[error("collections are not an array: {0:?}")]
    CollectionsAreNotAnArray(Value),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("tokio postgres error: {0}")]
    TokioPosgres(#[from] tokio_postgres::Error),

    #[error("timed out")]
    TimedOut,
}

impl<E> From<bb8::RunError<E>> for Error
where
    E: std::error::Error + 'static,
{
    fn from(err: bb8::RunError<E>) -> Self {
        match err {
            bb8::RunError::User(e) => Error::Bb8UserError(e.into()),
            bb8::RunError::TimedOut => Error::TimedOut,
        }
    }
}
