use crate::{Backend, Error, Page, Result};
use serde_json::Value;
use stac::{Catalog, Collection, Link};
use stac_api::{
    Collections, Conformance, Item, ItemCollection, Root, UrlBuilder, COLLECTIONS_URI, CORE_URI,
    FEATURES_URI, GEOJSON_URI, OGC_API_FEATURES_URI,
};

/// A structure for generating STAC API endpoints.
#[derive(Clone, Debug)]
pub struct Api<B: Backend> {
    /// The backend for this API.
    pub backend: B,

    catalog: Catalog,
    url_builder: UrlBuilder,
}

impl<B: Backend> Api<B>
where
    Error: From<<B as Backend>::Error>,
    Error: From<<<B as Backend>::Page as Page>::Error>,
{
    /// Creates a new endpoint generator with the given backend, catalog, and root url.
    ///
    /// The catalog is used as the root endpoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Catalog;
    /// use stac_api_backend::{Api, MemoryBackend};
    ///
    /// let api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// ```
    pub fn new(backend: B, catalog: Catalog, url: &str) -> Result<Api<B>> {
        Ok(Api {
            backend,
            catalog,
            url_builder: UrlBuilder::new(url)?,
        })
    }

    /// Returns the root endpoint, as defined by
    /// <https://github.com/radiantearth/stac-api-spec/tree/main/core#endpoints>.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Catalog;
    /// use stac_api_backend::{Api, MemoryBackend};
    ///
    /// let api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// # tokio_test::block_on(async {
    /// let root = api.root().await.unwrap();
    /// assert_eq!(root.catalog.id, "an-id");
    /// # });
    /// ```
    pub async fn root(&self) -> Result<Root> {
        let mut catalog = self.catalog.clone();
        catalog.links = vec![
            Link::root(self.url_builder.root()),
            Link::self_(self.url_builder.root()),
            Link::new(self.url_builder.conformance(), "conformance").json(),
            Link::new(self.url_builder.collections(), "data").json(),
        ];
        for collection in self.backend.collections().await? {
            catalog
                .links
                .push(Link::child(self.url_builder.collection(&collection.id)?))
        }
        Ok(Root {
            catalog,
            conformance: self.conformance(),
        })
    }

    /// Returns the conformance structure.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Catalog;
    /// use stac_api_backend::{Api, MemoryBackend, Backend};
    ///
    /// let api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// let conformance = api.conformance();
    /// ```
    pub fn conformance(&self) -> Conformance {
        Conformance {
            conforms_to: vec![
                CORE_URI.to_string(),
                FEATURES_URI.to_string(),
                COLLECTIONS_URI.to_string(),
                OGC_API_FEATURES_URI.to_string(),
                GEOJSON_URI.to_string(),
            ],
        }
    }

    /// Returns collections.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Catalog;
    /// use stac_api_backend::{Api, MemoryBackend, Backend};
    ///
    /// let api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// # tokio_test::block_on(async {
    /// let collections = api.collections().await.unwrap();
    /// # });
    /// ```
    pub async fn collections(&self) -> Result<Collections> {
        let mut collections = self.backend.collections().await?;
        for collection in &mut collections {
            self.add_collection_links(collection)?;
        }
        let links = vec![
            Link::root(self.url_builder.root()),
            Link::self_(self.url_builder.collections()),
        ];
        Ok(Collections {
            collections,
            links,
            additional_fields: Default::default(),
        })
    }

    /// Returns a collection or None.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Collection};
    /// use stac_api_backend::{Api, MemoryBackend, Backend};
    ///
    /// let mut api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// # tokio_test::block_on(async {
    /// assert_eq!(api.collection("collection-id").await.unwrap(), None);
    /// api.backend.add_collection(Collection::new("collection-id", "a description")).await.unwrap();
    /// assert_eq!(api.collection("collection-id").await.unwrap().unwrap().id, "collection-id");
    /// # });
    /// ```
    pub async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        if let Some(mut collection) = self.backend.collection(id).await? {
            self.add_collection_links(&mut collection)?;
            Ok(Some(collection))
        } else {
            Ok(None)
        }
    }

    /// Returns items.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Collection};
    /// use stac_api_backend::{Api, MemoryBackend, Backend};
    ///
    /// let mut api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// # tokio_test::block_on(async {
    /// api.backend.add_collection(Collection::new("collection-id", "a description")).await.unwrap();
    /// let item = api.items("collection-id", Default::default()).await.unwrap();
    /// # });
    /// ```
    pub async fn items(&self, id: &str, query: B::Query) -> Result<Option<ItemCollection>> {
        if let Some(page) = self.backend.items(id, query).await? {
            let url = self.url_builder.items(id)?;
            let mut item_collection = page.into_item_collection(url.clone())?;
            item_collection
                .links
                .push(Link::root(self.url_builder.root()));
            item_collection
                .links
                .push(Link::collection(self.url_builder.collection(id)?));
            item_collection.links.push(Link::self_(url).geojson());
            for item in &mut item_collection.items {
                self.add_item_links(item, id)?;
            }
            Ok(Some(item_collection))
        } else {
            Ok(None)
        }
    }

    /// Returns an item.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Collection, Item};
    /// use stac_api_backend::{Api, MemoryBackend, Backend};
    ///
    /// let mut api = Api::new(
    ///     MemoryBackend::new(),
    ///     Catalog::new("an-id", "a description"),
    ///     "http://stac-api-backend.test")
    /// .unwrap();
    /// # tokio_test::block_on(async {
    /// api.backend.add_collection(Collection::new("collection-id", "a description")).await.unwrap();
    /// api.backend.add_item(Item::new("item-id").collection("collection-id")).await.unwrap();
    /// let item = api.item("collection-id", "item-id").await.unwrap();
    /// # });
    /// ```
    pub async fn item(&self, collection_id: &str, id: &str) -> Result<Option<Item>> {
        if let Some(item) = self.backend.item(collection_id, id).await? {
            let mut item = item.try_into()?;
            self.add_item_links(&mut item, collection_id)?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn add_collection_links(&self, collection: &mut Collection) -> Result<()> {
        collection.links.push(Link::root(self.url_builder.root()));
        collection.links.push(Link::parent(self.url_builder.root()));
        collection
            .links
            .push(Link::self_(self.url_builder.collection(&collection.id)?));
        collection
            .links
            .push(Link::new(self.url_builder.items(&collection.id)?, "items").geojson());
        Ok(())
    }

    fn add_item_links(&self, item: &mut Item, collection_id: &str) -> Result<()> {
        let mut links = vec![
            serde_json::to_value(Link::root(self.url_builder.root()))?,
            serde_json::to_value(Link::parent(self.url_builder.collection(collection_id)?))?,
            serde_json::to_value(Link::collection(
                self.url_builder.collection(collection_id)?,
            ))?,
        ];
        if let Some(id) = item.get("id").and_then(|value| value.as_str()) {
            links.push(serde_json::to_value(
                Link::self_(self.url_builder.item(collection_id, id)?).geojson(),
            )?);
        }
        if let Some(existing_links) = item.get_mut("links").and_then(|value| value.as_array_mut()) {
            existing_links.extend(links);
        } else {
            let _ = item.insert("links".to_string(), Value::Array(links));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Api;
    use crate::{
        memory::{MemoryBackend, Query},
        Backend,
    };
    use stac::{Catalog, Collection, Item, Links, Validate};
    use stac_api::{COLLECTIONS_URI, CORE_URI, FEATURES_URI, GEOJSON_URI, OGC_API_FEATURES_URI};

    fn api() -> Api<MemoryBackend> {
        Api::new(
            MemoryBackend::new(),
            Catalog::new("test-catalog", "A catalog for testing"),
            "http://stac-api-backend.test",
        )
        .unwrap()
    }

    #[tokio::test]
    async fn root() {
        let root = api().root().await.unwrap();
        for uri in [
            CORE_URI,
            FEATURES_URI,
            COLLECTIONS_URI,
            OGC_API_FEATURES_URI,
            GEOJSON_URI,
        ] {
            assert!(
                root.conformance.conforms_to.contains(&uri.to_string()),
                "does not conform to {}",
                uri
            );
        }

        let catalog = root.catalog;
        catalog.clone().validate().unwrap();

        let root = catalog.link("root").unwrap();
        assert_eq!(root.href, "http://stac-api-backend.test/");
        assert_eq!(root.r#type.as_ref().unwrap(), "application/json");

        let self_link = catalog.link("self").unwrap();
        assert_eq!(self_link.href, "http://stac-api-backend.test/");
        assert_eq!(self_link.r#type.as_ref().unwrap(), "application/json");

        let conformance_link = catalog.link("conformance").unwrap();
        assert_eq!(
            conformance_link.href,
            "http://stac-api-backend.test/conformance"
        );
        assert_eq!(
            conformance_link.r#type.as_ref().unwrap(),
            "application/json"
        );

        let data_link = catalog.link("data").unwrap();
        assert_eq!(data_link.href, "http://stac-api-backend.test/collections");
        assert_eq!(
            conformance_link.r#type.as_ref().unwrap(),
            "application/json"
        );

        // TODO add service-desc, service-doc
    }

    #[tokio::test]
    async fn root_with_child() {
        let mut api = api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let root = api.root().await.unwrap();

        let links: Vec<_> = root.catalog.iter_child_links().collect();
        assert_eq!(links.len(), 1);

        let child = links[0];
        assert_eq!(child.href, "http://stac-api-backend.test/collections/an-id");
        assert_eq!(child.r#type.as_ref().unwrap(), "application/json");
    }

    #[tokio::test]
    async fn collections() {
        let mut api = api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let collections = api.collections().await.unwrap();
        assert_eq!(collections.collections.len(), 1);
        assert_eq!(
            collections.link("root").unwrap().href,
            "http://stac-api-backend.test/"
        );
        assert_eq!(
            collections.link("self").unwrap().href,
            "http://stac-api-backend.test/collections"
        );
    }

    #[tokio::test]
    async fn collection() {
        let mut api = api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let collection = api.collection("an-id").await.unwrap().unwrap();
        assert_eq!(
            collection.link("root").unwrap().href,
            "http://stac-api-backend.test/"
        );
        assert_eq!(
            collection.link("parent").unwrap().href,
            "http://stac-api-backend.test/"
        );
        assert_eq!(
            collection.link("self").unwrap().href,
            "http://stac-api-backend.test/collections/an-id"
        );
        assert_eq!(
            collection.link("items").unwrap().href,
            "http://stac-api-backend.test/collections/an-id/items"
        );
    }

    #[tokio::test]
    async fn items() {
        let mut api = api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let item = Item::new("item-id").collection("an-id");
        api.backend.add_items(vec![item]).await.unwrap();
        let items = api.items("an-id", Query::default()).await.unwrap().unwrap();
        assert_eq!(
            items.link("root").unwrap().href,
            "http://stac-api-backend.test/"
        );
        assert_eq!(
            items.link("self").unwrap().href,
            "http://stac-api-backend.test/collections/an-id/items"
        );
        assert_eq!(
            items.link("collection").unwrap().href,
            "http://stac-api-backend.test/collections/an-id"
        );
        let item: Item = items.items[0].clone().try_into().unwrap();
        assert_eq!(
            item.link("root").unwrap().href,
            "http://stac-api-backend.test/"
        );
        assert_eq!(
            item.link("parent").unwrap().href,
            "http://stac-api-backend.test/collections/an-id"
        );
        assert_eq!(
            item.link("collection").unwrap().href,
            "http://stac-api-backend.test/collections/an-id"
        );
        assert_eq!(
            item.link("self").unwrap().href,
            "http://stac-api-backend.test/collections/an-id/items/item-id"
        );
    }

    #[tokio::test]
    async fn item_paging() {
        let mut api = api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let item_a = Item::new("item-a").collection("an-id");
        let item_b = Item::new("item-b").collection("an-id");
        api.backend.add_items(vec![item_a, item_b]).await.unwrap();
        let items = api
            .items(
                "an-id",
                Query {
                    skip: Some(0),
                    take: Some(1),
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(items.items.len(), 1);
        assert_eq!(
            items.link("next").unwrap().href,
            "http://stac-api-backend.test/collections/an-id/items?skip=1&take=1"
        )
    }

    #[tokio::test]
    async fn item() {
        let mut api = api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let item = Item::new("an-item").collection("an-id");
        api.backend.add_item(item).await.unwrap();
        let _ = api.item("an-id", "an-item").await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn collection_404() {
        let api = api();
        assert_eq!(api.collection("an-id").await.unwrap(), None);
    }
}
