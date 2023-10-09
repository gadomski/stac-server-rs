// TODO document

use serde::Deserialize;
use stac::Value;
use stac_api_backend::Backend;
use std::{path::Path, str::FromStr};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
    task::JoinSet,
};

pub async fn load_hrefs<B>(backend: &mut B, hrefs: Vec<String>) -> Result<()>
where
    B: Backend,
    stac_api_backend::Error: From<B::Error>,
{
    // TODO this could probably be its own method on a backend?

    let mut join_set: JoinSet<Result<Value>> = JoinSet::new();
    for href in hrefs {
        join_set.spawn(async move { stac_async::read(href).await.map_err(Error::from) });
    }
    let mut item_vectors = Vec::new();
    while let Some(result) = join_set.join_next().await {
        let value = result.unwrap()?;
        match value {
            Value::Catalog(_) => return Err(Error::Load(value)),
            Value::Collection(collection) => {
                backend
                    .upsert_collection(collection)
                    .await
                    .map_err(stac_api_backend::Error::from)?;
            }
            Value::Item(item) => item_vectors.push(vec![item]),
            Value::ItemCollection(item_collection) => item_vectors.push(item_collection.items),
        }
    }
    for items in item_vectors {
        backend
            .add_items(items)
            .await
            .map_err(stac_api_backend::Error::from)?;
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("cannot load value")]
    Load(Value),

    #[error(transparent)]
    StacApiBackend(#[from] stac_api_backend::Error),

    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: stac_server::Config,

    // TODO document how to pick a backend with a config file
    #[serde(default = "BackendConfig::default")]
    pub backend: BackendConfig,
}

#[derive(Debug, Deserialize)]
pub enum BackendConfig {
    Memory,
    Pgstac(PgstacConfig),
}

#[derive(Debug, Deserialize)]
pub struct PgstacConfig {
    pub config: String,
}

impl Config {
    pub async fn from_toml(path: impl AsRef<Path>) -> Result<Config> {
        let mut reader = File::open(path).await.map(BufReader::new)?;
        let mut string = String::new();
        let _ = reader.read_to_string(&mut string).await?;
        string.parse()
    }
}

impl Default for Config {
    fn default() -> Self {
        let s = include_str!("config.toml");
        s.parse().unwrap()
    }
}

impl FromStr for Config {
    type Err = Error;
    fn from_str(s: &str) -> Result<Config> {
        toml::from_str(&s).map_err(Error::from)
    }
}

impl BackendConfig {
    pub fn set_pgstac_config(&mut self, config: impl ToString) {
        *self = BackendConfig::Pgstac(PgstacConfig {
            config: config.to_string(),
        })
    }
}

impl Default for BackendConfig {
    fn default() -> Self {
        BackendConfig::Memory
    }
}
