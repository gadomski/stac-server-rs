use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use stac::Collection;
use stac_backend::Backend;
use thiserror::Error;
use tokio_postgres::tls::NoTls;

#[derive(Clone, Debug)]
pub struct PgstacBackend {
    // TODO enable tls
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}

impl PgstacBackend {
    pub async fn from_str(s: &str) -> Result<PgstacBackend, Error> {
        let manager = PostgresConnectionManager::new_from_stringlike(s, NoTls)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(PgstacBackend { pool })
    }
}

#[async_trait]
impl Backend for PgstacBackend {
    async fn collections(&self) -> stac_backend::Result<Vec<Collection>> {
        // TODO don't unwrap
        let connection = self.pool.get().await.unwrap();
        let row = connection
            .query_one("SELECT * FROM pgstac.all_collections();", &[])
            .await
            .unwrap();
        let value = row.try_get(0).unwrap();
        Ok(serde_json::from_value(value).unwrap())
    }

    async fn add_collection(&mut self, _: Collection) -> stac_backend::Result<Option<Collection>> {
        unimplemented!()
    }
}
