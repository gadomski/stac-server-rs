use crate::Error;
use async_trait::async_trait;
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use serde_json::Value;
use stac::Collection;
use tokio_postgres::{Config, NoTls};

#[derive(Debug, Clone)]
pub struct Backend {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Backend {
    pub async fn new(config: Config) -> Result<Backend, Error> {
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = Pool::builder().build(manager).await.unwrap();
        Ok(Backend { pool })
    }

    async fn connection(
        &self,
    ) -> Result<
        PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        bb8::RunError<tokio_postgres::Error>,
    > {
        self.pool.get().await
    }
}

#[async_trait]
impl crate::backend::Backend for Backend {
    async fn collections(&self) -> Result<Vec<Collection>, Error> {
        let connection = self.connection().await?;
        let row = connection
            .query_one("SELECT pgstac.all_collections();", &[])
            .await?;
        let value: Value = row.try_get(0)?;
        if let Value::Array(values) = value {
            values
                .into_iter()
                .map(|value| serde_json::from_value(value).map_err(Error::from))
                .collect()
        } else {
            Err(Error::CollectionsAreNotAnArray(value))
        }
    }

    async fn collection(&self, id: &str) -> Result<Collection, Error> {
        let connection = self.connection().await?;
        let row = connection
            .query_one("SELECT * FROM pgstac.get_collection($1);", &[&id])
            .await?;
        let value: Value = row.try_get(0)?;
        serde_json::from_value(value).map_err(Error::from)
    }
}
