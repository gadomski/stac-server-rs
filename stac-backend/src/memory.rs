use crate::{Backend, Result};
use async_trait::async_trait;
use stac::Collection;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// An in-memory backend, mostly for testing.
#[derive(Clone, Debug)]
pub struct MemoryBackend {
    collections: Arc<RwLock<HashMap<String, Collection>>>,
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
        }
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    async fn collections(&self) -> Result<Vec<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.values().cloned().collect())
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.get(id).cloned())
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<Option<Collection>> {
        let mut collections = self.collections.write().unwrap();
        Ok(collections.insert(collection.id.clone(), collection))
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryBackend;
    use crate::Backend;
    use stac::{Catalog, Collection, Validate};
    use stac_api::LinkBuilder;

    fn link_builder() -> LinkBuilder {
        LinkBuilder::new(None)
    }

    fn catalog() -> Catalog {
        Catalog::new("an-id", "a description")
    }

    #[tokio::test]
    async fn root_endpoint() {
        let backend = MemoryBackend::new();
        let root = backend
            .root_endpoint(link_builder(), catalog())
            .await
            .unwrap();
        root.catalog.validate().unwrap();
    }

    #[tokio::test]
    async fn collections_endpoint_empty() {
        let backend = MemoryBackend::new();
        let collections = backend.collections_endpoint(link_builder()).await.unwrap();
        assert!(collections.collections.is_empty());
    }

    #[tokio::test]
    async fn collections_endpoint_with_one() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let collections = backend.collections_endpoint(link_builder()).await.unwrap();
        assert_eq!(collections.collections.len(), 1);
    }
}
