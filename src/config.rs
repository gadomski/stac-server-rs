use crate::Error;
use serde::Deserialize;
use stac::Catalog;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub browseable: bool,
    #[serde(rename = "ogc-api-features")]
    pub ogc_api_features: bool,
    pub catalog: CatalogConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CatalogConfig {
    pub id: String,
    pub description: String,
    pub title: Option<String>,
}

impl Config {
    pub fn from_toml<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
        let mut reader = File::open(path).map(BufReader::new)?;
        let mut config_string = String::new();
        reader.read_to_string(&mut config_string)?;
        toml::from_str(&config_string).map_err(Error::from)
    }

    pub fn conforms_to(&self) -> Vec<String> {
        let mut conforms_to = vec!["https://api.stacspec.org/v1.0.0-rc.2/core".to_string()];
        if self.browseable {
            conforms_to.push("https://api.stacspec.org/v1.0.0-rc.2/browseable".to_string());
        }
        if self.ogc_api_features {
            conforms_to.push("https://api.stacspec.org/v1.0.0-rc.2/ogcapi-features".to_string());
            conforms_to
                .push("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string());
            conforms_to
                .push("http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string());
        }
        conforms_to
    }
}

impl CatalogConfig {
    pub fn to_catalog(&self) -> Catalog {
        let mut catalog = Catalog::new(&self.id);
        catalog.description = self.description.clone();
        catalog.title = self.title.clone();
        catalog
    }
}
