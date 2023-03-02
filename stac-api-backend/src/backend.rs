use crate::PaginatedItemCollection;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use stac::{Collection, Item};
use std::error::Error;

/// A STAC API backend builds each STAC API endpoint.
#[async_trait]
pub trait Backend: Send + Sync + Clone {
    /// The error type returned by the backend.
    type Error: Error + From<stac_api::Error>;

    /// The structure of the pagination for this endpoint.
    type Pagination: DeserializeOwned + Serialize + Sync + Send;

    /// Returns collections.
    async fn collections(&self) -> Result<Vec<Collection>, Self::Error>;

    /// Returns a collection.
    async fn collection(&self, id: &str) -> Result<Option<Collection>, Self::Error>;

    /// Returns items.
    async fn items(
        &self,
        id: &str,
        pagination: Option<Self::Pagination>,
    ) -> Result<Option<PaginatedItemCollection<Self::Pagination>>, Self::Error>;

    /// Returns an item.
    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>, Self::Error>;

    /// Adds a collection to the backend.
    async fn add_collection(&mut self, collection: Collection) -> Result<(), Self::Error>;

    /// Adds an item to the backend.
    async fn add_item(&mut self, item: Item) -> Result<(), Self::Error>;
}
