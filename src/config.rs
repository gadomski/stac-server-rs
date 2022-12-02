use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub addr: String,
    pub catalog: CatalogConfig,
}

#[derive(Clone, Deserialize, Debug)]
pub struct CatalogConfig {
    pub id: String,
    pub description: String,
}
