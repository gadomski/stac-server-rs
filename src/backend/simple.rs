use crate::{Backend, Error};
use async_trait::async_trait;
use stac::{Collection, Item};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug)]
pub struct Simple {
    collections: Arc<RwLock<HashMap<String, CollectionWithItems>>>,
}

#[derive(Clone, Debug)]
struct CollectionWithItems {
    collection: Collection,
    items: Vec<Item>,
}

impl Simple {
    pub fn new() -> Simple {
        Simple {
            collections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Backend for Simple {
    async fn collections(&self) -> Result<Vec<Collection>, Error> {
        let collections = self.collections.read().unwrap(); // TODO can we not unwrap?
        Ok(collections.values().map(|c| c.collection.clone()).collect())
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>, Error> {
        let collections = self.collections.read().unwrap(); // TODO can we not unwrap?
        Ok(collections.get(id).map(|c| c.collection.clone()))
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<(), Error> {
        let mut collections = self.collections.write().unwrap(); // TODO can we not unwrap?
        collections.insert(
            collection.id.clone(),
            CollectionWithItems {
                collection,
                items: Vec::new(),
            },
        );
        Ok(())
    }

    async fn items(&self, collection_id: &str) -> Result<Vec<Item>, Error> {
        let collections = self.collections.read().unwrap(); // TODO can we not unwrap?
        let collection = collections
            .get(collection_id)
            .ok_or_else(|| Error::UnknownCollectionId(collection_id.to_string()))?;
        Ok(collection.items.clone())
    }

    async fn add_item(&mut self, mut item: Item) -> Result<(), Error> {
        // TODO test branches
        let mut collections = self.collections.write().unwrap(); // TODO can we not unwrap?
        if let Some(collection_id) = item.collection.take() {
            let collection = collections
                .get_mut(&collection_id)
                .ok_or_else(|| Error::UnknownCollectionId(collection_id.to_string()))?;
            collection.items.push(item);
            Ok(())
        } else if collections.len() == 1 {
            let (_, collection) = collections.iter_mut().next().unwrap();
            collection.items.push(item);
            Ok(())
        } else {
            Err(Error::NoCollection(item))
        }
    }
}
