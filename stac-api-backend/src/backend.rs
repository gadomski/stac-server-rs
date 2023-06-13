use crate::Page;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use stac::{Collection, Item};
use std::error::Error;

/// A STAC API backend builds each STAC API endpoint.
#[async_trait]
pub trait Backend: Send + Sync + Clone {
    /// The error type returned by the backend.
    type Error: Error;

    /// The type of the page returned by the items endpoint and by item search.
    type Page: Page;

    /// The type of the query used for the items endpoint.
    type Query: DeserializeOwned;

    /// Returns all collections in this backend.
    async fn collections(&self) -> Result<Vec<Collection>, Self::Error>;

    /// Returns a single collection.
    async fn collection(&self, id: &str) -> Result<Option<Collection>, Self::Error>;

    /// Returns items.
    async fn items(&self, id: &str, query: Self::Query) -> Result<Option<Self::Page>, Self::Error>;

    /// Returns an item.
    async fn item(&self, collection_id: &str, id: &str) -> Result<Option<Item>, Self::Error>;

    /// Adds a new collection to this backend.
    async fn add_collection(
        &mut self,
        collection: Collection,
    ) -> Result<Option<Collection>, Self::Error>;

    /// Adds new items to this backend.
    async fn add_items(&mut self, items: Vec<Item>) -> Result<(), Self::Error>;

    /// Adds a new item to this backend.
    async fn add_item(&mut self, item: Item) -> Result<(), Self::Error>;
}
