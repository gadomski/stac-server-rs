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

    /// The catalog configuration.
    pub catalog: CatalogConfig,
}

/// Catalog configuration.
#[derive(Clone, Debug, Deserialize)]
pub struct CatalogConfig {
    /// The catalog's id.
    pub id: String,

    /// The catalog's description.
    pub description: String,
}

impl CatalogConfig {
    /// Creates a new catalog from this catalog configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::CatalogConfig;
    /// let config = CatalogConfig { id: "an-id".to_string(), description: "a description".to_string() };
    /// let catalog = config.into_catalog();
    /// ```
    pub fn into_catalog(self) -> Catalog {
        Catalog::new(self.id, self.description)
    }
}
