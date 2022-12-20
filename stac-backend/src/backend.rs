use crate::Result;
use async_trait::async_trait;
use stac::Collection;

/// Trait for backends.
#[async_trait]
pub trait Backend: Send + Sync + Clone {
    /// Returns collections.
    async fn collections(&self) -> Result<Vec<Collection>>;

    /// Adds a collection to the backend.
    async fn add_collection(&mut self, collection: Collection) -> Result<Option<Collection>>;
}
