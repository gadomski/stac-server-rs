use std::num::TryFromIntError;

use crate::{Backend, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pgstac::{Client, Page, Search};
use serde::Deserialize;
use stac::{Collection, Item, Link};
use stac_api::{Item as ApiItem, ItemCollection, LinkBuilder};
use tokio_postgres::tls::NoTls;

const TOKEN_KEY: &str = "token";

#[derive(Clone, Debug)]
pub struct PgstacBackend {
    // TODO enable TLS
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Deserialize, Debug)]
pub struct Query {
    pub limit: Option<u64>,
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

    async fn items(
        &self,
        link_builder: LinkBuilder,
        id: &str,
        query: Query,
    ) -> Result<ItemCollection> {
        let connection = self.pool.get().await?;
        let client = Client::new(&*connection);
        let mut search = query.into_search()?;
        search.collections = vec![id.to_string()];
        let page = client.search(search).await?;
        item_collection(
            page,
            link_builder.next_items(id)?,
            link_builder.prev_items(id)?,
        )
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

impl Query {
    fn into_search(self) -> std::result::Result<Search, TryFromIntError> {
        let limit = if let Some(limit) = self.limit {
            Some(limit.try_into()?)
        } else {
            None
        };
        Ok(Search {
            token: self.token,
            limit: limit,
            ..Default::default()
        })
    }
}

fn item_collection(page: Page, mut next_link: Link, mut prev_link: Link) -> Result<ItemCollection> {
    // TODO can we avoid making the next link if we don't need it?
    // TODO once stac-api is released, we can move this into pgstac
    let mut links = Vec::with_capacity(2);
    if let Some(next) = page.next_token() {
        next_link.set_query_pair(TOKEN_KEY, &next)?;
        links.push(next_link);
    }
    if let Some(prev) = page.prev_token() {
        prev_link.set_query_pair(TOKEN_KEY, &prev)?;
        links.push(prev_link);
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
