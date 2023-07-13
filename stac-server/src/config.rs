use serde::Deserialize;
use stac::Catalog;

/// Server configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// The IP address of the server.
    pub addr: String,

    /// Should this server support features?
    ///
    /// Note that we don't allow just collections, because why.
    pub features: bool,

    /// The catalog that will serve as the landing page.
    pub catalog: Catalog,
}
