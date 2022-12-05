mod pgstac;
mod simple;

pub use {pgstac::Pgstac, simple::Simple};

use crate::Error;
use async_trait::async_trait;
use stac::{Collection, Item};

#[async_trait]
pub trait Backend: Clone + Send + Sync + std::fmt::Debug + 'static {
    async fn collections(&self) -> Result<Vec<Collection>, Error>;
    async fn collection(&self, id: &str) -> Result<Option<Collection>, Error>;
    async fn add_collection(&mut self, collection: Collection) -> Result<(), Error>;
    async fn items(&self, collection_id: &str) -> Result<Vec<Item>, Error>;
    async fn add_item(&mut self, item: Item) -> Result<(), Error>;
}
