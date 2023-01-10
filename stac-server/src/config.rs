//! Server configuration.

use crate::{Error, Result};
use serde::Deserialize;
use stac::Catalog;
use std::{path::Path, str::FromStr};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

/// stac-server-rs configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// The address to serve the application from.
    ///
    /// If not provided, this is assumed to be set elsewhere, e.g. via a command line interface.
    pub addr: Option<String>,

    /// Configure the landing page's attributes.
    pub catalog: CatalogConfig,
}

/// The landing page's attributes.
#[derive(Debug, Deserialize)]
pub struct CatalogConfig {
    /// The catalog id.
    pub id: String,

    /// The catalog description.
    pub description: String,
}

impl Config {
    /// Reads in configuration from a toml file.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::Config;
    /// # tokio_test::block_on(async {
    /// let config = Config::from_toml("data/config.toml").await.unwrap();
    /// # })
    /// ```
    pub async fn from_toml(path: impl AsRef<Path>) -> Result<Config> {
        let mut reader = File::open(path).await.map(BufReader::new)?;
        let mut string = String::new();
        reader.read_to_string(&mut string).await?;
        string.parse()
    }
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(s: &str) -> Result<Config> {
        toml::from_str(&s).map_err(Error::from)
    }
}

impl CatalogConfig {
    /// Converts this catalog config into a catalog.
    pub fn into_catalog(self) -> Catalog {
        Catalog::new(self.id, self.description)
    }
}
