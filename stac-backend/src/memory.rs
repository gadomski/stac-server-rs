use crate::{Backend, CollectionDoesNotExist, NoCollection, Result};
use async_trait::async_trait;
use stac::{Collection, Item};
use stac_api::ItemCollection;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use url::Url;

/// An in-memory backend, mostly for testing.
#[derive(Clone, Debug)]
pub struct MemoryBackend {
    collections: Arc<RwLock<HashMap<String, Collection>>>,
    items: Arc<RwLock<HashMap<String, HashMap<String, Item>>>>,
}

impl MemoryBackend {
    /// Creates a new memory backend.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_backend::MemoryBackend;
    /// let backend = MemoryBackend::new();
    /// ```
    pub fn new() -> MemoryBackend {
        MemoryBackend {
            collections: Arc::new(RwLock::new(HashMap::new())),
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    type Query = ();

    async fn collections(&self) -> Result<Vec<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.values().cloned().collect())
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.get(id).cloned())
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<()> {
        let mut collections = self.collections.write().unwrap();
        let _ = collections.insert(collection.id.clone(), collection);
        Ok(())
    }

    async fn items(&self, id: &str, _: Self::Query, _: Url) -> Result<ItemCollection> {
        let items = self.items.read().unwrap();
        if let Some(collection) = items.get(id) {
            let mut items = Vec::new();
            for item in collection.values() {
                items.push(item.clone().try_into()?);
            }
            let item_collection = ItemCollection::new(items)?;
            Ok(item_collection)
        } else {
            Err(Box::new(CollectionDoesNotExist(id.to_string())))
        }
    }

    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>> {
        let items = self.items.read().unwrap();
        Ok(items
            .get(collection_id)
            .and_then(|c| c.get(item_id))
            .cloned())
    }

    async fn add_item(&mut self, item: Item) -> Result<()> {
        if let Some(collection) = item.collection.as_ref().cloned() {
            if self.collection(&collection).await?.is_some() {
                let mut items = self.items.write().unwrap();
                let collection = items.entry(collection).or_default();
                collection.insert(item.id.clone(), item);
                Ok(())
            } else {
                Err(Box::new(CollectionDoesNotExist(collection)))
            }
        } else {
            Err(Box::new(NoCollection(item)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryBackend;
    use crate::backend::tests::test_suite;

    test_suite!(MemoryBackend, MemoryBackend::new());
}
