use crate::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use stac::{Catalog, Collection, Item};
use stac_api::{Collections, ItemCollection, LinkBuilder, Root};
use url::Url;

/// A STAC API backend builds each STAC API endpoint.
#[async_trait]
pub trait Backend: Send + Sync + Clone {
    type Query: DeserializeOwned + Sync + Send;

    /// Returns the root endpoint.
    ///
    /// The [Backend] trait provides a default implementation, but backends can
    /// override or extend that implementation as they want.
    async fn root_endpoint(&self, link_builder: LinkBuilder, mut catalog: Catalog) -> Result<Root> {
        let collections = self.collections().await?;
        let mut links = Vec::with_capacity(collections.len());
        for collection in collections {
            links.push(link_builder.child_collection(&collection)?);
        }
        catalog.links = links;
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

    async fn items_endpoint(
        &self,
        _: LinkBuilder,
        id: &str,
        query: Self::Query,
        url: Url,
    ) -> Result<ItemCollection> {
        self.items(id, query, url).await
    }

    async fn item_endpoint(
        &self,
        _: LinkBuilder,
        collection_id: &str,
        item_id: &str,
    ) -> Result<Option<Item>> {
        self.item(collection_id, item_id).await
    }

    /// Returns collections.
    async fn collections(&self) -> Result<Vec<Collection>>;

    /// Returns a collection.
    async fn collection(&self, id: &str) -> Result<Option<Collection>>;

    /// Returns items.
    async fn items(&self, id: &str, query: Self::Query, url: Url) -> Result<ItemCollection>;

    /// Returns an item.
    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>>;

    /// Adds a collection to the backend.
    async fn add_collection(&mut self, collection: Collection) -> Result<()>;

    /// Adds an item to the backend.
    async fn add_item(&mut self, item: Item) -> Result<()>;
}

#[cfg(all(test, any(feature = "pgstac", feature = "memory")))]
pub(crate) mod tests {
    macro_rules! test_suite {
        ($backend:ty, $body:expr) => {
            use crate::Backend;
            use stac::{media_type, Catalog, Collection, Item, Links, Validate};
            use stac_api::LinkBuilder;

            async fn backend() -> $backend {
                $body
            }

            fn link_builder() -> LinkBuilder {
                LinkBuilder::new("http://stac-backend.test".parse().unwrap())
            }

            fn catalog() -> Catalog {
                Catalog::new("a-catalog", "a description")
            }

            fn collection() -> Collection {
                Collection::new("a-collection", "a description")
            }

            fn item() -> Item {
                let mut item = Item::new("an-item");
                item.collection = Some("a-collection".to_string());
                item
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
            async fn root_endpoint_with_collections() {
                let mut backend = backend().await;
                backend.add_collection(collection()).await.unwrap();
                let root = backend
                    .root_endpoint(link_builder(), catalog())
                    .await
                    .unwrap();
                let catalog = root.catalog;
                let child_links: Vec<_> = catalog.iter_child_links().collect();
                assert_eq!(child_links.len(), 1);
                let child_link = child_links[0];
                assert_eq!(
                    child_link.href,
                    "http://stac-backend.test/collections/a-collection"
                );
                assert_eq!(child_link.r#type.as_ref().unwrap(), media_type::JSON);
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

            #[tokio::test]
            async fn item_endpoint_with_none() {
                let backend = backend().await;
                let item = backend
                    .item_endpoint(link_builder(), "a-collection", "an-item")
                    .await
                    .unwrap();
                assert!(item.is_none());
            }

            #[tokio::test]
            async fn item_endpoint_with_one() {
                let mut backend = backend().await;
                backend.add_collection(collection()).await.unwrap();
                backend.add_item(item()).await.unwrap();
                let item = backend
                    .item_endpoint(link_builder(), "a-collection", "an-item")
                    .await
                    .unwrap();
                assert!(item.is_some());
            }
        };
    }

    pub(crate) use test_suite;
}
