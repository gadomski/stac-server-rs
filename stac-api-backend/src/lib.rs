#![deny(unused_extern_crates)]

mod backend;
mod builder;
mod error;
#[cfg(feature = "memory")]
mod memory;
mod pagination_links;

#[cfg(feature = "memory")]
pub use self::memory::MemoryBackend;
pub use {
    backend::Backend,
    builder::Builder,
    error::Error,
    pagination_links::{PaginationLinks, UnresolvedLink},
};

pub type Result<T> = std::result::Result<T, Error>;
