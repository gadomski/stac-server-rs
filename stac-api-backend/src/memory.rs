use crate::{Backend, PaginationLinks, UnresolvedLink};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stac::{Collection, Item};
use stac_api::ItemCollection;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};
use thiserror::Error;

const DEFAULT_TAKE: usize = 20;
pub type Query = [(&'static str, usize); 2];

#[derive(Error, Debug)]
pub enum Error {
    #[error("collection with id {0} does not exist")]
    CollectionDoesNotExist(String),

    #[error("there is no collection on this item: {}", .0.id)]
    NoCollection(stac::Item),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),
}

#[derive(Clone, Debug)]
pub struct MemoryBackend {
    collections: Arc<RwLock<HashMap<String, Collection>>>,
    items: Arc<RwLock<HashMap<String, BTreeMap<String, Item>>>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pagination {
    pub skip: usize,
    pub take: usize,
}

impl MemoryBackend {
    pub fn new() -> MemoryBackend {
        MemoryBackend {
            collections: Arc::new(RwLock::new(HashMap::new())),
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Backend for MemoryBackend {
    type Error = Error;
    type Pagination = Pagination;
    type Query = Query;

    async fn collections(&self) -> Result<Vec<Collection>, Error> {
        let collections = self.collections.read().unwrap();
        Ok(collections.values().cloned().collect())
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>, Error> {
        let collections = self.collections.read().unwrap();
        Ok(collections.get(id).cloned())
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<(), Error> {
        let collection_id = collection.id.clone();
        {
            let mut collections = self.collections.write().unwrap();
            let _ = collections.insert(collection_id.clone(), collection); // TODO should this error if one exists?
        }
        let mut items = self.items.write().unwrap();
        // TODO what should happen if items already exist for the collection?
        items.insert(collection_id, BTreeMap::new());
        Ok(())
    }

    async fn items(
        &self,
        id: &str,
        pagination: Option<Pagination>,
    ) -> Result<Option<(ItemCollection, PaginationLinks<Self::Query>)>, Error> {
        let items = self.items.read().unwrap();
        if let Some(collection) = items.get(id) {
            let mut items = Vec::new();
            let pagination = pagination.unwrap_or_default();
            // TODO ceil page size
            for item in collection
                .values()
                .skip(pagination.skip)
                .take(pagination.take)
            {
                items.push(item.clone().try_into()?);
            }
            let item_collection = ItemCollection::new(items)?;
            Ok(Some((item_collection, pagination.links(collection.len()))))
        } else {
            Ok(None)
        }
    }

    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>, Error> {
        let items = self.items.read().unwrap();
        Ok(items
            .get(collection_id)
            .and_then(|c| c.get(item_id))
            .cloned())
    }

    async fn add_item(&mut self, item: Item) -> Result<(), Error> {
        if let Some(collection) = item.collection.as_ref().cloned() {
            if self.collection(&collection).await?.is_some() {
                let mut items = self.items.write().unwrap();
                let collection = items.entry(collection).or_default();
                collection.insert(item.id.clone(), item);
                Ok(())
            } else {
                Err(Error::CollectionDoesNotExist(collection))
            }
        } else {
            Err(Error::NoCollection(item))
        }
    }
}

impl From<Error> for crate::Error {
    fn from(err: Error) -> crate::Error {
        crate::Error::Backend(Box::new(err))
    }
}

impl Pagination {
    fn links(&self, n: usize) -> PaginationLinks<Query> {
        PaginationLinks {
            next: self.next(n).map(|p| p.to_unresolved_link()),
            prev: self.prev().map(|p| p.to_unresolved_link()),
        }
    }

    fn next(&self, n: usize) -> Option<Pagination> {
        if self.skip + self.take < n {
            Some(Pagination {
                skip: self.skip + self.take,
                take: self.take,
            })
        } else {
            None
        }
    }

    fn prev(&self) -> Option<Pagination> {
        if self.skip >= self.take {
            Some(Pagination {
                skip: self.skip - self.take,
                take: self.take,
            })
        } else {
            None
        }
    }

    fn to_unresolved_link(&self) -> UnresolvedLink<Query> {
        UnresolvedLink::new(self.to_query())
    }

    fn to_query(&self) -> Query {
        [("skip", self.skip), ("take", self.take)]
    }
}

impl Default for Pagination {
    fn default() -> Pagination {
        Pagination {
            skip: 0,
            take: DEFAULT_TAKE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MemoryBackend, Pagination};
    use crate::Backend;
    use stac::{Collection, Item};

    #[tokio::test]
    async fn adding_collection_adds_items_entry() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let (items, _) = backend.items("an-id", None).await.unwrap().unwrap();
        assert!(items.features.is_empty());
    }

    #[tokio::test]
    async fn paginate_items() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        for i in 0..10 {
            backend
                .add_item(Item::new(format!("item-{}", i)).collection("an-id"))
                .await
                .unwrap();
        }
        let ids: Vec<_> = backend
            .items("an-id", Some(Pagination { take: 2, skip: 3 }))
            .await
            .unwrap()
            .unwrap()
            .0
            .features
            .into_iter()
            .map(|item| Item::try_from(item).unwrap().id)
            .collect();
        assert_eq!(ids, vec!["item-3", "item-4"]);
    }
}