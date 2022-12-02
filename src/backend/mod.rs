mod pgstac;

pub use pgstac::Backend as PgstacBackend;

use crate::Error;
use async_trait::async_trait;
use stac::Collection;

#[async_trait]
pub trait Backend: Clone {
    async fn collections(&self) -> Result<Vec<Collection>, Error>;
    async fn collection(&self, id: &str) -> Result<Collection, Error>;
}
