use crate::Result;
use async_trait::async_trait;
use stac::{Catalog, Collection};
use stac_api::{Collections, LinkBuilder, Root};

/// A STAC API backend builds each STAC API endpoint.
#[async_trait]
pub trait Backend: Send + Sync + Clone {
    /// Returns the root endpoint.
    ///
    /// The [Backend] trait provides a default implementation, but backends can
    /// override or extend that implementation as they want.
    async fn root_endpoint(&self, _: LinkBuilder, catalog: Catalog) -> Result<Root> {
        Ok(Root {
            catalog,
            conforms_to: Vec::new(),
        })
    }

    async fn collections_endpoint(&self, _: LinkBuilder) -> Result<Collections> {
        Ok(Collections {
            collections: self.collections().await?,
            links: Vec::new(),
        })
    }

    async fn collection_endpoint(&self, _: LinkBuilder, id: &str) -> Result<Option<Collection>> {
        self.collection(id).await
    }

    /// Returns collections.
    async fn collections(&self) -> Result<Vec<Collection>>;

    /// Returns a collection.
    async fn collection(&self, id: &str) -> Result<Option<Collection>>;

    /// Adds a collection to the backend.
    async fn add_collection(&mut self, collection: Collection) -> Result<Option<Collection>>;
}
