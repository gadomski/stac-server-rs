use crate::Config;
use stac::Catalog;
use stac_api::Backend;

/// Shared state for the API.
#[derive(Clone, Debug)]
pub struct State<B: Backend> {
    /// The backend.
    pub backend: B,

    /// The socket address that the server is being served on.
    pub addr: Option<String>,

    /// The root catalog, used to build the landing page.
    pub catalog: Catalog,
}

impl<B: Backend> State<B> {
    /// Creates a new state from a backend and a config.
    pub fn new(backend: B, config: Config) -> State<B> {
        State {
            backend,
            addr: config.addr,
            catalog: config.catalog.into_catalog(),
        }
    }
}
