use super::Api;
use crate::{Backend, Error, Items, Result};
use http::Method;
use serde_json::Value;
use stac::{Collection, Item, Link};
use stac_api::{Collections, ItemCollection};

impl<B> Api<B>
where
    B: Backend,
    Error: From<<B as Backend>::Error>,
{
    /// Returns collections.
    pub async fn collections(&self) -> Result<Collections> {
        // TODO collection pagination
        // https://github.com/radiantearth/stac-api-spec/tree/release/v1.0.0/ogcapi-features#collection-pagination
        let mut collections = self.backend.collections().await?;
        for collection in &mut collections {
            collection.links.extend([
                Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::parent(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::self_(self.url_builder.collection(&collection.id)?)
                    .title(collection.title.clone()),
                Link::new(self.url_builder.items(&collection.id)?, "items")
                    .title("Items".to_string()),
            ]);
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
    pub async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        if let Some(mut collection) = self.backend.collection(id).await? {
            collection.links.extend([
                Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::parent(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::self_(self.url_builder.collection(&collection.id)?)
                    .title(collection.title.clone()),
                Link::new(self.url_builder.items(&collection.id)?, "items")
                    .title("Items".to_string())
                    .geojson(),
            ]);
            Ok(Some(collection))
        } else {
            Ok(None)
        }
    }

    /// Returns items.
    pub async fn items(&self, id: &str, items: Items<B::Paging>) -> Result<Option<ItemCollection>> {
        if let Some(page) = self.backend.items(id, items.clone()).await? {
            let mut url = self.url_builder.items(id)?;

            let get_items = stac_api::GetItems::try_from(items.items)?;
            let query = serde_urlencoded::to_string(&get_items)?;
            if !query.is_empty() {
                url.set_query(Some(&query));
            }
            let mut item_collection =
                page.into_item_collection(&url, &Method::GET, items.paging)?;
            item_collection.links.extend([
                Link::root(self.url_builder.root()).title(self.catalog.title.clone()),
                Link::collection(self.url_builder.collection(id)?),
            ]);

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
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use super::super::tests;
    use crate::{assert_link, memory::Paging, Backend, Items};
    use stac::{Collection, Item, Links};
    use stac_validate::Validate;

    #[tokio::test]
    async fn root_links_with_features() {
        let mut api = tests::api();
        api.features = true;
        let root = api.root().await.unwrap();

        assert_link!(
            root.catalog,
            "data",
            "http://stac-api-backend.test/collections",
            "application/json"
        );
        assert_link!(
            root.catalog,
            "conformance",
            "http://stac-api-backend.test/conformance",
            "application/json"
        );
    }

    #[tokio::test]
    async fn root_links_without_features() {
        let mut api = tests::api();
        api.features = false;
        let root = api.root().await.unwrap();
        assert!(root.catalog.link("data").is_none());
        assert!(root.catalog.link("conformance").is_none());
    }

    #[tokio::test]
    async fn collections_links() {
        let collections = tests::api().collections().await.unwrap();
        assert_link!(
            collections,
            "root",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            collections,
            "self",
            "http://stac-api-backend.test/collections",
            "application/json"
        );
    }

    #[tokio::test]
    async fn collections() {
        let mut api = tests::api();
        assert!(api.collections().await.unwrap().collections.is_empty());
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        assert_eq!(api.collections().await.unwrap().collections.len(), 1);
    }

    #[tokio::test]
    async fn collection_miss() {
        assert!(tests::api().collection("id").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn collection() {
        let mut api = tests::api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let collection = api.collection("an-id").await.unwrap().unwrap();
        assert_link!(
            collection,
            "root",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            collection,
            "parent",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            collection,
            "self",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );
        assert_link!(
            collection,
            "items",
            "http://stac-api-backend.test/collections/an-id/items",
            "application/geo+json"
        );
        collection.validate().unwrap();
    }

    #[tokio::test]
    async fn items_miss() {
        let mut api = tests::api();
        assert!(api
            .items("an-id", Items::default())
            .await
            .unwrap()
            .is_none());
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        assert!(api
            .items("an-id", Items::default())
            .await
            .unwrap()
            .unwrap()
            .items
            .is_empty());

        let item = Item::new("item-id").collection("an-id");
        api.backend.add_item(item).await.unwrap();

        let items = api.items("an-id", Items::default()).await.unwrap().unwrap();

        assert_link!(
            items,
            "root",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            items,
            "self",
            "http://stac-api-backend.test/collections/an-id/items",
            "application/geo+json"
        );
        assert_link!(
            items,
            "collection",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );

        let item: Item = items.items[0].clone().try_into().unwrap();
        assert_link!(
            item,
            "root",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            item,
            "parent",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );
        assert_link!(
            item,
            "collection",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );
        assert_link!(
            item,
            "self",
            "http://stac-api-backend.test/collections/an-id/items/item-id",
            "application/geo+json"
        );
        item.validate().unwrap();
    }

    #[tokio::test]
    async fn item_paging() {
        let mut api = tests::api();
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
        assert_link!(
            items,
            "next",
            "http://stac-api-backend.test/collections/an-id/items?skip=1&take=1",
            "application/geo+json"
        )
    }

    #[tokio::test]
    async fn item() {
        let mut api = tests::api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let item = Item::new("item-id").collection("an-id");
        api.backend.add_item(item).await.unwrap();
        let item = api.item("an-id", "item-id").await.unwrap().unwrap();
        assert_link!(
            item,
            "root",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            item,
            "parent",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );
        assert_link!(
            item,
            "collection",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );
        assert_link!(
            item,
            "self",
            "http://stac-api-backend.test/collections/an-id/items/item-id",
            "application/geo+json"
        );
        item.validate().unwrap();
    }
}
