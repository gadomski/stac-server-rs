use crate::{Items, Page};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use stac::{Collection, Item};
use std::fmt::Debug;

/// A STAC API backend builds each STAC API endpoint.
#[async_trait]
pub trait Backend: Send + Sync + Clone + 'static {
    /// The error type returned by the backend.
    type Error: std::error::Error;

    /// The paging object.
    ///
    /// Some might use a token, some might use a skip+take, some might do something else.
    type Paging: Debug + Clone + Serialize + Default + DeserializeOwned + Send + Sync;

    /// Returns all collections in this backend.
    async fn collections(&self) -> Result<Vec<Collection>, Self::Error>;

    /// Returns a single collection.
    async fn collection(&self, id: &str) -> Result<Option<Collection>, Self::Error>;

    /// Returns items.
    async fn items(
        &self,
        id: &str,
        items: Items<Self::Paging>,
    ) -> Result<Option<Page<Self::Paging>>, Self::Error>;

    /// Returns an item.
    async fn item(&self, collection_id: &str, id: &str) -> Result<Option<Item>, Self::Error>;

    /// Adds a new collection to this backend.
    async fn add_collection(
        &mut self,
        collection: Collection,
    ) -> Result<Option<Collection>, Self::Error>;

    /// Adds or updates a collection in this backend.
    async fn upsert_collection(
        &mut self,
        collection: Collection,
    ) -> Result<Option<Collection>, Self::Error>;

    /// Deletes a collection and its items.
    async fn delete_collection(&mut self, id: &str) -> Result<(), Self::Error>;

    /// Adds new items to this backend.
    async fn add_items(&mut self, items: Vec<Item>) -> Result<(), Self::Error>;

    /// Adds or updates items in this backend.
    async fn upsert_items(&mut self, items: Vec<Item>) -> Result<(), Self::Error>;

    /// Adds a new item to this backend.
    async fn add_item(&mut self, item: Item) -> Result<(), Self::Error>;
}
