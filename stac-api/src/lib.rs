mod backend;

#[cfg(feature = "pgstac")]
pub use backend::PgstacBackend;
pub use backend::{Backend, MemoryBackend};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    Bb8TokioPostgresRun(#[from] bb8::RunError<tokio_postgres::Error>),

    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),

    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}
