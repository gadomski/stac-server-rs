mod backend;
mod memory;

pub use {backend::Backend, memory::MemoryBackend};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("connection error: {0}")]
    Connection(String),
}
