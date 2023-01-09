#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    Bb8TokioPostgresRun(#[from] bb8::RunError<tokio_postgres::Error>),

    /// std::io::Error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),

    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),

    /// toml::de::Error
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
