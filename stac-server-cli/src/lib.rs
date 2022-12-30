use anyhow::Result;
use stac::Value;
use stac_backend::{Backend, MemoryBackend};
use stac_server::Config;
use std::path::PathBuf;

pub async fn load_files_into_memory_backend(
    backend: &mut MemoryBackend,
    paths: &[PathBuf],
) -> Result<()> {
    use Value::*;
    let mut collections = Vec::new();
    let mut items = Vec::new();
    for path in paths {
        match stac_async::read(path.to_string_lossy()).await? {
            Item(item) => items.push(item),
            ItemCollection(item_collection) => items.extend(item_collection.items),
            Collection(collection) => collections.push(collection),
            Catalog(_) => {} // TODO notify, warn, something
        }
    }
    for collection in collections {
        backend.add_collection(collection).await?;
    }
    // TODO add items
    Ok(())
}

pub fn default_config() -> Config {
    let s = include_str!("../data/config.toml");
    s.parse().unwrap()
}