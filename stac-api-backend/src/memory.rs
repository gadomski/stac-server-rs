use crate::{Backend, Items, Page};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stac::{Collection, Item, Links};
use stac_api::ItemCollection;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
use thiserror::Error;

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

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Paging {
    /// The number of items to skip.
    pub skip: Option<usize>,

    /// The number of items to return.
    pub take: Option<usize>,
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
    type Paging = Paging;

    async fn collections(&self) -> Result<Vec<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.values().cloned().collect())
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let collections = self.collections.read().unwrap();
        Ok(collections.get(id).cloned())
    }

    async fn items(&self, id: &str, query: Items<Paging>) -> Result<Option<Page<Paging>>> {
        let skip = query.paging.skip.unwrap_or(0);
        let mut take = query.paging.take.unwrap_or(self.take);
        if let Some(limit) = query.items.limit {
            let limit: usize = limit.try_into()?;
            if limit < take {
                take = limit;
            }
        }
        let items = self.items.read().unwrap();
        if let Some(items) = items.get(id) {
            let bbox = query
                .items
                .bbox
                .as_ref()
                .map(|bbox| stac::geo::bbox(bbox))
                .transpose()?;
            let datetime = query
                .items
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
                .collect();
            let number_matched = items.len();
            let items = items
                .into_iter()
                .cloned()
                .skip(skip)
                .take(take)
                .map(|item| item.try_into().map_err(Error::from))
                .collect::<Result<_>>()?;
            let mut item_collection = ItemCollection::new(items)?;
            item_collection.number_matched = Some(number_matched.try_into()?);
            let next = if skip + take < number_matched {
                Some(Paging {
                    skip: Some(skip + take),
                    take: Some(take),
                })
            } else {
                None
            };
            let prev = if skip > 0 {
                if skip >= take {
                    Some(Paging {
                        skip: Some(skip - take),
                        take: Some(take),
                    })
                } else {
                    Some(Paging {
                        skip: None,
                        take: Some(take),
                    })
                }
            } else {
                None
            };
            Ok(Some(Page {
                item_collection,
                next,
                prev,
            }))
        } else {
            let collections = self.collections.read().unwrap();
            if collections.contains_key(id) {
                let mut item_collection = ItemCollection::new(vec![])?;
                item_collection.number_matched = Some(0);
                Ok(Some(Page {
                    item_collection,
                    next: None,
                    prev: None,
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
        collection.remove_structural_links();
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
