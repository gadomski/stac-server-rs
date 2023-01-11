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
    async fn add_collection(&mut self, collection: Collection) -> Result<()>;
}

#[cfg(all(test, any(feature = "pgstac", feature = "memory")))]
pub(crate) mod tests {
    macro_rules! test_suite {
        ($backend:ty, $body:expr) => {
            use crate::Backend;
            use stac::{Catalog, Collection, Validate};
            use stac_api::LinkBuilder;

            async fn backend() -> $backend {
                $body
            }

            fn link_builder() -> LinkBuilder {
                LinkBuilder::new(None)
            }

            fn catalog() -> Catalog {
                Catalog::new("a-catalog", "a description")
            }

            fn collection() -> Collection {
                Collection::new("a-collection", "a description")
            }

            #[tokio::test]
            async fn root_endpoint() {
                let backend = backend().await;
                let root = backend
                    .root_endpoint(link_builder(), catalog())
                    .await
                    .unwrap();
                root.catalog.validate().unwrap();
            }

            #[tokio::test]
            async fn collections_endpoint_with_none() {
                let backend = backend().await;
                let collections = backend.collections_endpoint(link_builder()).await.unwrap();
                assert!(collections.collections.is_empty());
            }

            #[tokio::test]
            async fn collections_endpoint_with_one() {
                let mut backend = backend().await;
                backend.add_collection(collection()).await.unwrap();
                let collections = backend.collections_endpoint(link_builder()).await.unwrap();
                assert_eq!(collections.collections.len(), 1);
            }

            #[tokio::test]
            async fn collection_endpoint_with_none() {
                let backend = backend().await;
                let collection = backend
                    .collection_endpoint(link_builder(), "a-collection")
                    .await
                    .unwrap();
                assert!(collection.is_none());
            }

            #[tokio::test]
            async fn collection_endpoint_with_one() {
                let mut backend = backend().await;
                backend.add_collection(collection()).await.unwrap();
                let collection = backend
                    .collection_endpoint(link_builder(), "a-collection")
                    .await
                    .unwrap();
                assert!(collection.is_some());
            }
        };
    }

    pub(crate) use test_suite;
}
