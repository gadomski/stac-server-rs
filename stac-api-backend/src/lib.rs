#![deny(unused_extern_crates)]

mod backend;
mod builder;
mod error;
#[cfg(feature = "memory")]
mod memory;

#[cfg(feature = "memory")]
pub use self::memory::MemoryBackend;
pub use {
    backend::{Backend, PaginationLinks},
    builder::Builder,
    error::Error,
};

pub type Result<T> = std::result::Result<T, Error>;
