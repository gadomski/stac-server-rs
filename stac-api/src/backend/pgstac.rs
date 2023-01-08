use crate::{Backend, Error, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pgstac::Client;
use stac::Collection;
use tokio_postgres::tls::NoTls;

#[derive(Clone, Debug)]
pub struct PgstacBackend {
    // TODO enable TLS
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PgstacBackend {
    pub async fn from_str(s: &str) -> Result<PgstacBackend> {
        let manager = PostgresConnectionManager::new_from_stringlike(s, NoTls)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(PgstacBackend { pool })
    }
}

#[async_trait]
impl Backend for PgstacBackend {
    async fn collections(&self) -> Result<Vec<Collection>> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.collections().await.map_err(Error::from)
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.collection(id).await.map_err(Error::from)
    }

    async fn add_collection(&mut self, _: Collection) -> Result<Option<Collection>> {
        unimplemented!()
    }
}
