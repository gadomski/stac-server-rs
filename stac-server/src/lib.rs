//! STAC API server implementation using [axum](https://github.com/tokio-rs/axum).

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

mod config;
mod error;
mod router;

pub use {
    config::{CatalogConfig, Config},
    error::Error,
    router::api,
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
use tokio_test as _;

/// Start a server.
pub async fn serve<B>(backend: B, config: Config) -> Result<()>
where
    B: stac_api_backend::Backend + 'static,
    stac_api_backend::Error: From<<B as stac_api_backend::Backend>::Error>,
    stac_api_backend::Error:
        From<<<B as stac_api_backend::Backend>::Page as stac_api_backend::Page>::Error>,
{
    let addr = config.addr.parse::<std::net::SocketAddr>()?;
    let mut open_api = aide::openapi::OpenApi {
        info: aide::openapi::Info {
            description: Some(config.catalog.description.clone()),
            ..aide::openapi::Info::default()
        },
        ..aide::openapi::OpenApi::default()
    };
    let api = api(backend, config)?;
    axum::Server::bind(&addr)
        .serve(
            api.finish_api(&mut open_api)
                .layer(axum::Extension(open_api))
                .into_make_service(),
        )
        .await
        .map_err(Error::from)
}
