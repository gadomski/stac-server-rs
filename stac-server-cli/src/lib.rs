use stac::Value;
use stac_backend::Backend;
use stac_server::Config;

pub async fn load_files_into_backend<B>(backend: &mut B, hrefs: &[String])
where
    B: Backend,
{
    use Value::*;
    let mut collections = Vec::new();
    let mut items = Vec::new();
    for href in hrefs {
        match stac_async::read(href).await.unwrap() {
            Item(item) => items.push(item),
            ItemCollection(item_collection) => items.extend(item_collection.items),
            Collection(collection) => collections.push(collection),
            Catalog(_) => {} // TODO notify, warn, something
        }
    }
    for collection in collections {
        backend.add_collection(collection).await.unwrap();
    }
    // TODO add items
}

pub fn default_config() -> Config {
    let s = include_str!("../data/config.toml");
    s.parse().unwrap()
}
