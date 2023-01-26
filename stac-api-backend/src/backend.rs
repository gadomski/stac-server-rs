use crate::PaginationLinks;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use stac::{Collection, Item};
use stac_api::ItemCollection;
use std::error::Error;

/// A STAC API backend builds each STAC API endpoint.
#[async_trait]
pub trait Backend: Send + Sync + Clone {
    type Error: Error + From<stac_api::Error>;
    type Pagination: DeserializeOwned + Sync + Send;
    type Query: Serialize;

    /// Returns collections.
    async fn collections(&self) -> Result<Vec<Collection>, Self::Error>;

    /// Returns a collection.
    async fn collection(&self, id: &str) -> Result<Option<Collection>, Self::Error>;

    /// Returns items.
    async fn items(
        &self,
        id: &str,
        pagination: Option<Self::Pagination>,
    ) -> Result<Option<(ItemCollection, PaginationLinks<Self::Query>)>, Self::Error>;

    /// Returns an item.
    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>, Self::Error>;

    /// Adds a collection to the backend.
    async fn add_collection(&mut self, collection: Collection) -> Result<(), Self::Error>;

    /// Adds an item to the backend.
    async fn add_item(&mut self, item: Item) -> Result<(), Self::Error>;
}