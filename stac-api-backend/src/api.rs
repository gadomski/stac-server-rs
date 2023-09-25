use crate::{Backend, Error, GetItems, Items, Result};
use serde_json::Value;
use stac::{Catalog, Collection, Item, Link};
use stac_api::{
    Collections, Conformance, ItemCollection, Root, UrlBuilder, COLLECTIONS_URI, CORE_URI,
    FEATURES_URI, GEOJSON_URI, OGC_API_FEATURES_URI,
};

/// The default media type for the `service-desc` links.
pub const DEFAULT_SERVICE_DESC_MEDIA_TYPE: &str = "application/vnd.oai.openapi+json;version=3.1";

/// A structure for generating STAC API endpoints.
#[derive(Clone, Debug)]
pub struct Api<B: Backend> {
    /// The backend for this API.
    pub backend: B,

    /// The url builder for this api.
    pub url_builder: UrlBuilder,

    /// If true, this API will include links for the [Features](https://github.com/radiantearth/stac-api-spec/tree/main/ogcapi-features) endpoints.
    ///
    /// We don't support _just_ collections.
    pub features: bool,

    /// The media type for the `service-desc` endpoint.
    ///
    /// Defaults to [DEFAULT_SERVICE_DESC_MEDIA_TYPE].
    pub service_desc_media_type: String,

    catalog: Catalog,
}

impl<B: Backend> Api<B>
where
    Error: From<<B as Backend>::Error>,
{
    /// Creates a new endpoint generator with the given backend, catalog, and root url.
    ///
    /// The catalog is used as the root endpoint. By default, the API will
    /// include links for
    /// [Features](https://github.com/radiantearth/stac-api-spec/tree/main/ogcapi-features)
    /// -- set `features` to `False` to disable this behavior.
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
    ///     "http://stac-api-backend.test",)
    /// .unwrap();
    /// assert!(api.features);
    /// ```
    pub fn new(backend: B, catalog: Catalog, url: &str) -> Result<Api<B>> {
        Ok(Api {
            backend,
            catalog,
            features: true,
            service_desc_media_type: DEFAULT_SERVICE_DESC_MEDIA_TYPE.to_string(),
            url_builder: UrlBuilder::new(url)?,
        })
    }

    /// Returns the [root endpoint](https://github.com/radiantearth/stac-api-spec/tree/main/core#endpoints).
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
        catalog.links.extend([
            Link::root(self.url_builder.root()),
            Link::self_(self.url_builder.root()),
            Link::new(self.url_builder.conformance(), "conformance")
                .json()
                .title("Conformance".to_string()),
            Link::new(self.url_builder.service_desc(), "service-desc")
                .r#type(self.service_desc_media_type.clone()),
            Link::new(
                format!("{}.html", self.url_builder.service_desc()),
                "service-doc",
            )
            .r#type("text/html".to_string()),
        ]);
        if self.features {
            catalog.links.push(
                Link::new(self.url_builder.collections(), "data")
                    .json()
                    .title("Collections".to_string()),
            );
        }
        for collection in self.backend.collections().await? {
            catalog.links.push(
                Link::child(self.url_builder.collection(&collection.id)?).title(collection.title),
            )
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
        let mut conforms_to = vec![CORE_URI.to_string()];
        if self.features {
            conforms_to.extend([
                FEATURES_URI.to_string(),
                COLLECTIONS_URI.to_string(),
                OGC_API_FEATURES_URI.to_string(),
                GEOJSON_URI.to_string(),
            ])
        }
        Conformance { conforms_to }
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
            Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
            Link::self_(self.url_builder.collections()).title("Collections".to_string()),
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
    pub async fn items(&self, id: &str, items: Items<B::Paging>) -> Result<Option<ItemCollection>> {
        if let Some(page) = self.backend.items(id, items.clone()).await? {
            let url = self.url_builder.items(id)?;
            let mut self_url = url.clone();
            let get_items =
                stac_api::GetItems::try_from(items.items).map(|get_items| GetItems {
                    get_items,
                    paging: items.paging,
                })?;
            let query = serde_qs::to_string(&get_items)?;
            if !query.is_empty() {
                self_url.set_query(Some(&query));
            }
            let mut item_collection = page.item_collection;
            item_collection.links.extend([
                Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::collection(self.url_builder.collection(id)?),
                Link::self_(self_url).geojson(),
            ]);
            if let Some(next) = page.next {
                let mut url = url.clone();
                let mut get_items = get_items.clone();
                get_items.paging = next;
                url.set_query(Some(&serde_qs::to_string(&get_items)?));
                item_collection.links.push(Link::new(url, "next").geojson());
            }
            if let Some(prev) = page.prev {
                let mut url = url.clone();
                let mut get_items = get_items.clone();
                get_items.paging = prev;
                url.set_query(Some(&serde_qs::to_string(&get_items)?));
                item_collection.links.push(Link::new(url, "prev").geojson());
            }
            for item in &mut item_collection.items {
                let mut links = vec![
                    serde_json::to_value(
                        Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
                    )?,
                    serde_json::to_value(Link::parent(self.url_builder.collection(id)?))?,
                    serde_json::to_value(Link::collection(self.url_builder.collection(id)?))?,
                ];
                if let Some(item_id) = item.get("id").and_then(|value| value.as_str()) {
                    links.push(serde_json::to_value(
                        Link::self_(self.url_builder.item(id, item_id)?).geojson(),
                    )?);
                }
                if let Some(existing_links) =
                    item.get_mut("links").and_then(|value| value.as_array_mut())
                {
                    existing_links.extend(links);
                } else {
                    let _ = item.insert("links".to_string(), Value::Array(links));
                }
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
        if let Some(mut item) = self.backend.item(collection_id, id).await? {
            let collection_url = self.url_builder.collection(collection_id)?;
            item.links.extend([
                Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::parent(collection_url.clone()),
                Link::collection(collection_url),
                Link::self_(self.url_builder.item(collection_id, id)?).geojson(),
            ]);
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn add_collection_links(&self, collection: &mut Collection) -> Result<()> {
        collection
            .links
            .push(Link::root(self.url_builder.root()).title(self.catalog.title.clone()));
        collection
            .links
            .push(Link::parent(self.url_builder.root()).title(self.catalog.title.clone()));
        collection.links.push(
            Link::self_(self.url_builder.collection(&collection.id)?)
                .title(collection.title.clone()),
        );
        collection.links.push(
            Link::new(self.url_builder.items(&collection.id)?, "items")
                .geojson()
                .title("Items".to_string()),
        );
        Ok(())
    }
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use super::Api;
    use crate::{
        memory::{MemoryBackend, Paging},
        Backend, Items,
    };
    use stac::{Catalog, Collection, Item, Links};
    use stac_api::{COLLECTIONS_URI, CORE_URI, FEATURES_URI, GEOJSON_URI, OGC_API_FEATURES_URI};
    use stac_validate::Validate;

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
        let items = api.items("an-id", Items::default()).await.unwrap().unwrap();
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
        let mut items: Items<Paging> = Items::default();
        items.paging.skip = Some(0);
        items.paging.take = Some(1);
        let items = api.items("an-id", items).await.unwrap().unwrap();
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

    #[tokio::test]
    async fn no_feature() {
        let mut api = api();
        api.features = false;
        let root = api.root().await.unwrap();
        assert_eq!(root.catalog.link("data"), None);
    }
}
