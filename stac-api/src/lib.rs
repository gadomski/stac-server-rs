mod backend;
mod config;
pub mod endpoint;
mod error;
mod hrefs;

#[cfg(feature = "pgstac")]
pub use backend::PgstacBackend;
pub use {
    backend::{Backend, MemoryBackend},
    config::{CatalogConfig, Config},
    error::Error,
    hrefs::Hrefs,
};

pub type Result<T> = std::result::Result<T, Error>;
