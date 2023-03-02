//! An opinionated backend for STAC API servers.
//!
//! The [STAC API specification](https://github.com/radiantearth/stac-api-spec)
//! describes how [STAC](https://github.com/radiantearth/stac-spec) objects
//! should be served over the network. This crate defines an interface for
//! fetching STAC objects from storage, and providing them to a STAC API server.
//!
//! The goal of this crate is to provide an abstraction layer between actual
//! server implementations, which might vary from framework to framework, and
//! their backends. This crate is **opinionated** because it sacrifices
//! flexibility in favor of enforcing a certain interface. We feel comfortable
//! with these opinionated decisions because we have implemented this interface
//! twice (as of this writing): once for an [in-memory backend provided as a
//! feature in this crate](crate::memory::MemoryBackend), and again for a
//! backend based on [pgstac](https://github.com/stac-utils/pgstac).

#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

mod backend;
mod endpoints;
mod error;
#[cfg(feature = "memory")]
mod memory;
mod paginated_item_collection;

#[cfg(feature = "memory")]
pub use self::memory::MemoryBackend;
pub use {
    backend::Backend, endpoints::Endpoints, error::Error,
    paginated_item_collection::PaginatedItemCollection,
};

/// A crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
use tokio as _;
