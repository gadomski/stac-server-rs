//! STAC API backend for pgstac.

#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pgstac::Client;
use stac::{Collection, Item, Link};
use stac_api::{ItemCollection, Items};
use stac_api_backend::Backend;
use thiserror::Error;
use tokio_postgres::tls::NoTls;
use url::Url;

/// The pgstac backend.
#[derive(Clone, Debug)]
pub struct PgstacBackend {
    pool: Pool<PostgresConnectionManager<NoTls>>, // TODO allow tls
}

/// Crate-specific error enum.
#[derive(Error, Debug)]
pub enum Error {
    /// [bb8::RunError]
    #[error(transparent)]
    Bb8TokioPostgresRun(#[from] bb8::RunError<tokio_postgres::Error>),

    /// [pgstac::Error]
    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// [tokio_postgres::Error]
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A page of results.
#[derive(Debug)]
pub struct Page(pgstac::Page);

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

    async fn items(&self, id: &str, items: Items) -> Result<Option<Self::Page>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        let search = items.into_search(id);
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

    async fn delete_collection(&mut self, id: &str) -> Result<()> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.delete_collection(id).await?;
        Ok(())
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
            let _ = url.query_pairs_mut().append_pair("token", &next);
            links.push(Link::new(url, "next").geojson());
        }
        if let Some(prev) = self.0.prev_token() {
            let mut url = url.clone();
            let _ = url.query_pairs_mut().append_pair("token", &prev);
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
