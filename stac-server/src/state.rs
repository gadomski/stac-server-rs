use crate::{Config, Result};
use stac::Catalog;
use stac_api::Backend;
use url::Url;

/// Shared state for the API.
#[derive(Clone, Debug)]
pub struct State<B: Backend> {
    /// The backend.
    pub backend: B,

    /// The root url of the server.
    ///
    /// Should only be None for test servers.
    pub root: Option<Url>,

    /// The root catalog, used to build the landing page.
    pub catalog: Catalog,
}

impl<B: Backend> State<B> {
    /// Creates a new state from a backend and a config.
    pub fn new(backend: B, config: Config) -> Result<State<B>> {
        // TODO enable https roots
        let root = if let Some(addr) = config.addr {
            Some(Url::parse(&format!("http://{}", addr))?)
        } else {
            None
        };
        Ok(State {
            backend,
            root,
            catalog: config.catalog.into_catalog(),
        })
    }
}
