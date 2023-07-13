use axum::Server;
use futures_util::stream::StreamExt;
use geojson::{Geometry, Value};
use stac::{Collection, Item};
use stac_api::Items;
use stac_api_backend::{Backend, Error, MemoryBackend, PgstacBackend};
use stac_async::ApiClient;
use stac_server::{CatalogConfig, Config};
use std::net::TcpListener;

#[tokio::test]
async fn memory() {
    test(MemoryBackend::new()).await
}

#[tokio::test]
#[ignore = "pgstac test skipped by default, because it requires external services"]
async fn pgstac() {
    let config = "postgresql://username:password@localhost:5432/postgis";
    let (_, _) = tokio_postgres::connect(config, tokio_postgres::NoTls)
        .await
        .unwrap();
    test(PgstacBackend::connect(config).await.unwrap()).await
}

async fn test<B>(mut backend: B)
where
    B: Backend + 'static,
    Error: From<<B as Backend>::Error>,
    <B as Backend>::Paging: Send + Sync,
{
    if let Some(_) = backend.collection("collection-id").await.unwrap() {
        backend.delete_collection("collection-id").await.unwrap();
    }
    backend
        .add_collection(Collection::new("collection-id", "A test collection"))
        .await
        .unwrap();
    let mut items = Vec::new();
    for i in 0..10 {
        let mut item = Item::new(format!("item-{}", i)).collection("collection-id");
        item.properties.datetime = Some(format!("2023-07-{:02}T00:00:00Z", i + 1));
        item.set_geometry(Geometry::new(Value::Polygon(vec![vec![
            vec![-105.0, 40.0 + f64::from(i)],
            vec![-104.0, 40.0 + f64::from(i)],
            vec![-104.0, 41.0 + f64::from(i)],
            vec![-105.0, 41.0 + f64::from(i)],
            vec![-105.0, 40.0 + f64::from(i)],
        ]])))
        .unwrap();
        items.push(item);
    }
    backend.add_items(items).await.unwrap();
    let config = Config {
        addr: "127.0.0.1:7822".to_string(),
        catalog: CatalogConfig {
            id: "test-catalog".to_string(),
            description: "A test catalog".to_string(),
        },
    };

    let listener = TcpListener::bind(&config.addr).unwrap();
    let api = stac_server::api(backend, config).unwrap();
    let builder = Server::from_tcp(listener).unwrap();
    let server = builder.serve(api.into_make_service());
    tokio::spawn(async { server.await.unwrap() });

    let client = ApiClient::new("http://127.0.0.1:7822").unwrap();
    let _ = client.collection("collection-id").await.unwrap().unwrap();
    assert_eq!(client.collection("not-an-id").await.unwrap(), None);

    let items: Vec<_> = client
        .items("collection-id", None)
        .await
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
        .await;
    assert_eq!(items.len(), 10);

    let items: Vec<_> = client
        .items(
            "collection-id",
            Items {
                limit: Some(2),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
        .await;
    assert_eq!(items.len(), 10);

    let items: Vec<_> = client
        .items(
            "collection-id",
            Items {
                bbox: Some(vec![-110.0, 43.5, -100.0, 45.5]),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
        .await;
    assert_eq!(items.len(), 3);

    let items: Vec<_> = client
        .items(
            "collection-id",
            Items {
                datetime: Some("2023-07-02T00:00:00Z/2023-07-04T00:00:00Z".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .map(|result| result.unwrap())
        .collect()
        .await;
    assert_eq!(items.len(), 3);
}
