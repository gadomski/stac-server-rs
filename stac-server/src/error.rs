use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// [std::io::Error]
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// [toml::de::Error]
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
