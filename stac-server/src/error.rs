use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
pub enum Error {
    /// std::io::Error
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// stac_async::Error
    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    /// stac_backend::Error
    #[error(transparent)]
    StacBackend(#[from] stac_backend::Error),

    /// toml::de::Error
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
}
