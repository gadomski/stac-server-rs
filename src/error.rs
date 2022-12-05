use stac::Item;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("addr parse error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("bb8 user error: {0}")]
    Bb8UserError(Box<dyn std::error::Error>),

    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("no collection on item: {0:?}")]
    NoCollection(Item),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("timed out")]
    TimedOut,

    #[error("tokio postgres error: {0}")]
    TokioPostgres(#[from] tokio_postgres::Error),

    #[error("toml de error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("unknown collection id: {0}")]
    UnknownCollectionId(String),
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
