use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pgstac::Client;
use schemars::JsonSchema;
use serde::Deserialize;
use stac::{Collection, Item, Link};
use stac_api::{ItemCollection, Search};
use stac_api_backend::Backend;
use thiserror::Error;
use tokio_postgres::tls::NoTls;
use url::Url;

#[derive(Clone, Debug)]
pub struct PgstacBackend {
    pool: Pool<PostgresConnectionManager<NoTls>>, // TODO allow tls
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Bb8TokioPostgresRun(#[from] bb8::RunError<tokio_postgres::Error>),

    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Page(pgstac::Page);

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Query {
    token: Option<String>,
}

impl PgstacBackend {
    /// Creates a new pgstac backend.
    pub async fn connect(config: &str) -> Result<PgstacBackend> {
        let manager = PostgresConnectionManager::new_from_stringlike(config, NoTls)?;
        let pool = Pool::builder().build(manager).await?;
        Ok(PgstacBackend { pool })
    }
}

#[async_trait]
impl Backend for PgstacBackend {
    type Error = Error;
    type Page = Page;
    type Query = Query;

    async fn collections(&self) -> Result<Vec<Collection>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.collections().await.map_err(Error::from)
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.collection(id).await.map_err(Error::from)
    }

    async fn items(&self, id: &str, query: Query) -> Result<Option<Self::Page>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        let mut search = Search {
            collections: Some(vec![id.to_string()]),
            ..Default::default()
        };
        if let Some(token) = query.token {
            search
                .additional_fields
                .insert("token".to_string(), token.into());
        }
        let page = client.search(search).await?;
        if page.features.is_empty() && client.collection(id).await?.is_none() {
            Ok(None)
        } else {
            Ok(Some(Page(page)))
        }
    }

    async fn item(&self, collection_id: &str, id: &str) -> Result<Option<Item>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.item(id, collection_id).await.map_err(Error::from)
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<Option<Collection>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.add_collection(collection).await?;
        Ok(None) // TODO check and retrieve the previous collection
    }

    async fn upsert_collection(&mut self, collection: Collection) -> Result<Option<Collection>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.upsert_collection(collection).await?;
        Ok(None) // TODO check and retrieve the previous collection
    }

    async fn add_items(&mut self, items: Vec<Item>) -> Result<()> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.add_items(&items).await.map_err(Error::from)
    }

    async fn upsert_items(&mut self, items: Vec<Item>) -> Result<()> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.upsert_items(&items).await.map_err(Error::from)
    }

    async fn add_item(&mut self, item: Item) -> Result<()> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.add_item(item).await.map_err(Error::from)
    }
}

impl stac_api_backend::Page for Page {
    type Error = Error;
    fn into_item_collection(self, url: Url) -> Result<ItemCollection> {
        let mut links = vec![];
        if let Some(next) = self.0.next_token() {
            let mut url = url.clone();
            url.query_pairs_mut().append_pair("token", &next);
            links.push(Link::new(url, "next").geojson());
        }
        if let Some(prev) = self.0.prev_token() {
            let mut url = url.clone();
            url.query_pairs_mut().append_pair("token", &prev);
            links.push(Link::new(url, "prev").geojson());
        }
        let mut item_collection = ItemCollection::new(self.0.features)?;
        item_collection.links = links;
        Ok(item_collection)
    }
}

impl From<Error> for stac_api_backend::Error {
    fn from(value: Error) -> Self {
        stac_api_backend::Error::Backend(Box::new(value))
    }
}
