use crate::{Backend, Error};
use async_trait::async_trait;
use bb8::{Pool, PooledConnection, RunError};
use bb8_postgres::PostgresConnectionManager;
use serde::de::DeserializeOwned;
use stac::{Collection, Item};
use tokio_postgres::{
    types::{Json, ToSql},
    Config, NoTls, ToStatement,
};

#[derive(Clone, Debug)]
pub struct Pgstac {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Pgstac {
    pub async fn new(config: Config) -> Result<Pgstac, Error> {
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = Pool::builder().build(manager).await?;
        Ok(Pgstac { pool })
    }

    async fn connection(
        &self,
    ) -> Result<
        PooledConnection<'_, PostgresConnectionManager<NoTls>>,
        RunError<tokio_postgres::Error>,
    > {
        self.pool.get().await
    }

    async fn query_one<T: ?Sized + ToStatement, D: DeserializeOwned>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<D, Error> {
        let connection = self.connection().await?;
        let row = connection.query_one(statement, params).await?;
        let value = row.try_get(0)?;
        serde_json::from_value(value).map_err(Error::from)
    }
}

#[async_trait]
impl Backend for Pgstac {
    async fn collections(&self) -> Result<Vec<Collection>, Error> {
        self.query_one("SELECT * FROM pgstac.all_collections();", &[])
            .await
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>, Error> {
        // TODO deal with 404s
        self.query_one("SELECT * FROM pgstac.get_collection($1);", &[&id])
            .await
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<(), Error> {
        self.query_one(
            "SELECT * FROM pgstac.create_collection($1::text::jsonb)",
            &[&Json(collection)],
        )
        .await
    }

    async fn items(&self, collection_id: &str) -> Result<Vec<Item>, Error> {
        unimplemented!()
    }

    async fn add_item(&mut self, item: Item) -> Result<(), Error> {
        unimplemented!()
    }
}
