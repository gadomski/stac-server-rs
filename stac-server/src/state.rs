use crate::{Config, Result};
use stac::Catalog;
use stac_backend::Backend;
use url::Url;

/// Shared state for the API.
#[derive(Clone, Debug)]
pub struct State<B: Backend> {
    /// The backend.
    pub backend: B,

    /// The root url of the server.
    pub root: Url,

    /// The root catalog, used to build the landing page.
    pub catalog: Catalog,
}

impl<B: Backend> State<B> {
    /// Creates a new state from a backend and a config.
    pub fn new(backend: B, config: Config) -> Result<State<B>> {
        // TODO enable https roots
        let root = Url::parse(&format!("http://{}", config.addr))?;
        Ok(State {
            backend,
            root,
            catalog: config.catalog.into_catalog(),
        })
    }
}
