mod backend;
#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "pgstac")]
mod pgstac;

#[cfg(feature = "memory")]
pub use self::memory::MemoryBackend;
#[cfg(feature = "pgstac")]
pub use self::pgstac::PgstacBackend;
pub use backend::Backend;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
#[error("the item with id {} does not have a collection", .0.id)]
pub struct NoCollection(stac::Item);

#[derive(thiserror::Error, Debug)]
#[error("collection with id {0} does not exist")]
pub struct CollectionDoesNotExist(String);
