#![deny(unused_extern_crates)]

mod config;
mod error;
mod router;

pub use {
    config::{CatalogConfig, Config},
    error::Error,
    router::api,
};

pub type Result<T> = std::result::Result<T, Error>;
