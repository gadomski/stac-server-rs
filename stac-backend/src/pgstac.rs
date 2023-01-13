use crate::{Backend, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pgstac::{Client, Page, Search};
use serde::Deserialize;
use stac::{media_type, Collection, Item, Link};
use stac_api::{Item as ApiItem, ItemCollection};
use tokio_postgres::tls::NoTls;
use url::Url;

const TOKEN_KEY: &str = "token";

#[derive(Clone, Debug)]
pub struct PgstacBackend {
    // TODO enable TLS
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Deserialize, Debug)]
pub struct Query {
    pub token: Option<String>,
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
    type Query = Query;

    async fn collections(&self) -> Result<Vec<Collection>> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.collections().await.map_err(Box::from)
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.collection(id).await.map_err(Box::from)
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<()> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.add_collection(collection).await.map_err(Box::from)
    }

    async fn items(&self, id: &str, query: Query, url: Url) -> Result<ItemCollection> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        let search = Search {
            collections: vec![id.to_string()],
            token: query.token,
            ..Default::default()
        };
        let page = client.search(search).await?;
        item_collection(page, url)
    }

    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.item(item_id, collection_id).await.map_err(Box::from)
    }

    async fn add_item(&mut self, item: Item) -> Result<()> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        client.add_item(item).await.map_err(Box::from)
    }
}

fn item_collection(page: Page, url: Url) -> Result<ItemCollection> {
    // TODO once stac-api is released, we can move this into pgstac
    let mut links = Vec::with_capacity(2);
    if let Some(next) = page.next_token() {
        let mut link = Link::new(set_token(&url, &next), "next");
        link.r#type = Some(media_type::GEOJSON.to_string());
        links.push(link);
    }
    if let Some(prev) = page.prev_token() {
        let mut link = Link::new(set_token(&url, &prev), "prev");
        link.r#type = Some(media_type::GEOJSON.to_string());
        links.push(link);
    }
    let number_matched = if let Some(matched) = page.context.matched {
        Some(matched.try_into()?)
    } else {
        None
    };
    let number_returned = if let Some(returned) = page.context.returned {
        Some(returned.try_into()?)
    } else {
        None
    };
    Ok(ItemCollection {
        r#type: page.r#type,
        features: page
            .features
            .into_iter()
            .map(|item| ApiItem(item))
            .collect(),
        links,
        number_matched: number_matched,
        number_returned: number_returned,
    })
}

fn set_token(url: &Url, value: &str) -> Url {
    let mut new_url = url.clone();
    new_url
        .query_pairs_mut()
        .clear()
        .extend_pairs(url.query_pairs().filter(|(key, _)| key != TOKEN_KEY))
        .append_pair(TOKEN_KEY, value);
    new_url
}
