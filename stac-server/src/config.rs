//! Server configuration.

use crate::{Error, Result};
use serde::Deserialize;
use stac::Catalog;
use std::{path::Path, str::FromStr};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_addr")]
    pub addr: String,

    pub catalog: CatalogConfig,
}

#[derive(Debug, Deserialize)]
pub struct CatalogConfig {
    pub id: String,

    pub description: String,
}

impl Config {
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
    pub fn into_catalog(self) -> Catalog {
        Catalog::new(self.id, self.description)
    }
}

fn default_addr() -> String {
    "localhost:7822".to_string()
}
