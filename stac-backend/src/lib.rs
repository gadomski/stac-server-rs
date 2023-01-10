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
