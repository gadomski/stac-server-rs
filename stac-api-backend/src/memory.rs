use crate::Backend;
use async_trait::async_trait;
use stac::{Collection, Item, Link, Links};
use stac_api::{ItemCollection, Items};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use thiserror::Error;
use url::Url;

const DEFAULT_TAKE: usize = 20;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no collection id={0}")]
    CollectionNotFound(String),

    #[error("no collection set on item with id={}", .0.id)]
    NoCollection(Item),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    Stac(#[from] stac::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    #[error(transparent)]
    TryInt(#[from] std::num::TryFromIntError),
}

type Result<T> = std::result::Result<T, Error>;

/// A backend that stores its collections and items in memory.
///
/// Used mostly for testing.
#[derive(Clone, Debug)]
pub struct MemoryBackend {
    collections: Arc<RwLock<BTreeMap<String, Collection>>>,
    items: Arc<RwLock<BTreeMap<String, Vec<Item>>>>,
    take: usize,
}

/// A page from the memory backend.
#[derive(Debug)]
pub struct Page {
    items: Vec<Item>,
    number_matched: usize,
    skip: usize,
    take: usize,
}

impl MemoryBackend {
    /// Creates a new memory backend.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api_backend::MemoryBackend;
    /// let backend = MemoryBackend::new();
    /// ```
    pub fn new() -> MemoryBackend {
        MemoryBackend {
            collections: Arc::new(RwLock::new(BTreeMap::new())),
            items: Arc::new(RwLock::new(BTreeMap::new())),
            take: DEFAULT_TAKE,
        }
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    type Error = Error;
    type Page = Page;

    async fn collections(&self) -> Result<Vec<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.values().cloned().collect())
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.get(id).cloned())
    }

    async fn items(&self, id: &str, query: Items) -> Result<Option<Page>> {
        let skip = query
            .additional_fields
            .get("skip")
            .and_then(|s| s.as_str())
            .map(|s| s.parse())
            .unwrap_or(Ok(0))?;
        let mut take = query
            .additional_fields
            .get("take")
            .and_then(|s| s.as_str())
            .map(|s| s.parse())
            .unwrap_or(Ok(self.take))?;
        if let Some(limit) = query.limit {
            let limit: usize = limit.try_into()?;
            if limit < take {
                take = limit;
            }
        }
        let items = self.items.read().unwrap();
        if let Some(items) = items.get(id) {
            let number_matched = items.len();
            let bbox = query
                .bbox
                .as_ref()
                .map(|bbox| stac::geo::bbox(bbox))
                .transpose()?;
            let datetime = query
                .datetime
                .as_ref()
                .map(|datetime| stac::datetime::parse(datetime))
                .transpose()?;
            let items: Vec<_> = items
                .iter()
                .filter(|item| {
                    bbox.map(|bbox| item.intersects_bbox(bbox).unwrap_or(false))
                        .unwrap_or(true)
                        && datetime
                            .map(|(start, end)| {
                                item.intersects_datetimes(start, end).unwrap_or(false)
                            })
                            .unwrap_or(true)
                })
                .cloned()
                .skip(skip)
                .take(take)
                .collect();
            Ok(Some(Page {
                items,
                number_matched,
                skip,
                take,
            }))
        } else {
            let collections = self.collections.read().unwrap();
            if collections.contains_key(id) {
                Ok(Some(Page {
                    items: vec![],
                    number_matched: 0,
                    skip,
                    take,
                }))
            } else {
                Ok(None)
            }
        }
    }

    async fn item(&self, collection_id: &str, id: &str) -> Result<Option<Item>> {
        let items = self.items.read().unwrap();
        if let Some(item) = items
            .get(collection_id)
            .and_then(|items| items.iter().find(|item| item.id == id))
        {
            Ok(Some(item.clone()))
        } else {
            Ok(None)
        }
    }

    async fn add_collection(&mut self, mut collection: Collection) -> Result<Option<Collection>> {
        collection.remove_structural_links(); // TODO should we handle this at the API layer?
        let mut collections = self.collections.write().unwrap(); // TODO handle poison gracefully
        Ok(collections.insert(collection.id.clone(), collection))
    }

    async fn upsert_collection(&mut self, collection: Collection) -> Result<Option<Collection>> {
        self.add_collection(collection).await
    }

    async fn delete_collection(&mut self, id: &str) -> Result<()> {
        {
            let mut items = self.items.write().unwrap();
            let _ = items.remove(id);
        }
        {
            let mut collections = self.collections.write().unwrap();
            if collections.contains_key(id) {
                let _ = collections.remove(id);
                Ok(())
            } else {
                Err(Error::CollectionNotFound(id.to_string()))
            }
        }
    }

    async fn add_items(&mut self, items: Vec<Item>) -> Result<()> {
        let collections = self.collections.read().unwrap();
        let mut items_map = self.items.write().unwrap();
        for mut item in items {
            if let Some(collection) = item.collection.clone() {
                if collections.contains_key(&collection) {
                    item.remove_structural_links();
                    items_map.entry(collection.clone()).or_default().push(item);
                } else {
                    return Err(Error::CollectionNotFound(collection.clone()));
                }
            } else {
                return Err(Error::NoCollection(item));
            }
        }
        Ok(())
    }

    async fn upsert_items(&mut self, items: Vec<Item>) -> Result<()> {
        self.add_items(items).await
    }

    async fn add_item(&mut self, item: Item) -> Result<()> {
        self.add_items(vec![item]).await
    }
}

impl crate::Page for Page {
    type Error = Error;

    fn into_item_collection(self, url: Url) -> Result<ItemCollection> {
        let items = self
            .items
            .into_iter()
            .map(|item| item.try_into().map_err(Error::from))
            .collect::<Result<Vec<stac_api::Item>>>()?;
        let mut links = Vec::new();
        if items.len() == self.take {
            let mut url = url.clone();
            let _ = url
                .query_pairs_mut()
                .append_pair("skip", &(self.skip + items.len()).to_string())
                .append_pair("take", &self.take.to_string());
            links.push(Link::new(url, "next").geojson());
        }
        if self.skip > 0 {
            let skip = if self.skip > self.take {
                self.skip - self.take
            } else {
                0
            };
            let mut url = url.clone();
            let _ = url
                .query_pairs_mut()
                .append_pair("skip", &skip.to_string())
                .append_pair("take", &self.take.to_string());
            links.push(Link::new(url, "prev").geojson());
        }
        let mut item_collection = ItemCollection::new(items)?;
        item_collection.number_matched = Some(self.number_matched.try_into()?);
        item_collection.links = links;
        Ok(item_collection)
    }
}

impl From<Error> for crate::Error {
    fn from(value: Error) -> Self {
        crate::Error::Backend(Box::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryBackend;
    use crate::Backend;
    use stac::Collection;

    #[tokio::test]
    async fn add_collection() {
        let mut backend = MemoryBackend::new();
        let _ = backend
            .add_collection(Collection::new("a-collection", "A description"))
            .await
            .unwrap();
        assert_eq!(backend.collections().await.unwrap().len(), 1);
    }
}
