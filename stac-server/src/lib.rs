//! stac-server
//!
//! A [STAC API server](https://github.com/radiantearth/stac-api-spec) written in Rust.

#![deny(missing_docs)]

mod error;
mod extract;
mod router;
mod state;

pub use {error::Error, router::api, state::State};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;
