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

impl Config {
    /// The root url for this config.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::Config;
    /// let mut config = Config::default();
    /// config.addr = "stac-server-rs.test/stac/v1".to_string();
    /// assert_eq!(config.root_url(), "http://stac-server-rs.test/stac/v1");
    /// ```
    pub fn root_url(&self) -> String {
        // TODO enable https? Maybe?
        format!("http://{}", self.addr)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            addr: "127.0.0.1:7822".to_string(),
            features: true,
            catalog: Catalog::new(
                "stac-server-rs",
                "The default STAC API server from stac-server-rs",
            ),
        }
    }
}
