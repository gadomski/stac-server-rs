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
    /// use stac_api::MemoryBackend;
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

    async fn add_collection(&mut self, collection: Collection) -> Result<Option<Collection>> {
        let mut collections = self.collections.write().unwrap();
        Ok(collections.insert(collection.id.clone(), collection))
    }
}
