use crate::Config;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use stac::{Collection, Item};
use stac_api::{Collections, Conformance, ItemCollection, Root};
use stac_api_backend::{Backend, Builder};

pub fn api<B: Backend + 'static>(backend: B, config: Config) -> crate::Result<Router>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    let link_builder = format!("http://{}", config.addr).parse()?; // TODO enable https
    let catalog = config.catalog.into_catalog();
    let builder = Builder::new(backend, catalog, link_builder);
    Ok(Router::new()
        .route("/", get(root))
        .route("/conformance", get(conformance))
        .route("/collections", get(collections))
        .route("/collections/:collection_id", get(collection))
        .route("/collections/:collection_id/items", get(items))
        .route("/collections/:collection_id/items/:item_id", get(item))
        // TODO add search
        // TODO add queryables
        .with_state(builder))
}

async fn root<B: Backend>(
    State(builder): State<Builder<B>>,
) -> Result<Json<Root>, impl IntoResponse>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    builder
        .root()
        .await
        .map(Json)
        .map_err(internal_server_error)
}

async fn conformance<B: Backend>(State(builder): State<Builder<B>>) -> Json<Conformance>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    Json(builder.conformance())
}

async fn collections<B: Backend>(
    State(builder): State<Builder<B>>,
) -> Result<Json<Collections>, impl IntoResponse>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    builder
        .collections()
        .await
        .map(Json)
        .map_err(internal_server_error)
}

pub async fn collection<B: Backend>(
    State(builder): State<Builder<B>>,
    Path(id): Path<String>,
) -> Result<Json<Collection>, impl IntoResponse>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    builder
        .collection(&id)
        .await
        .map_err(internal_server_error)
        .and_then(|option| {
            if let Some(value) = option {
                Ok(Json(value))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    format!("no collection with id={}", id),
                ))
            }
        })
}

pub async fn items<B: Backend>(
    State(builder): State<Builder<B>>,
    Path(id): Path<String>,
    pagination: Option<Query<B::Pagination>>,
) -> Result<Json<ItemCollection>, impl IntoResponse>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    let pagination = pagination.map(|Query(p)| p);
    builder
        .items(&id, pagination)
        .await
        .map_err(internal_server_error)
        .and_then(|option| {
            if let Some(value) = option {
                Ok(Json(value))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    format!("no collection with id={}", id),
                ))
            }
        })
}

pub async fn item<B: Backend>(
    State(builder): State<Builder<B>>,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Json<Item>, impl IntoResponse>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    builder
        .item(&id, &item_id)
        .await
        .map_err(internal_server_error)
        .and_then(|option| {
            if let Some(value) = option {
                Ok(Json(value))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    format!("no item with id={} in collection={}", item_id, id),
                ))
            }
        })
}

fn internal_server_error(err: stac_api_backend::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("internal server error: {}", err),
    )
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use axum::{
        body::{Body, HttpBody},
        http::{Request, StatusCode},
    };
    use stac::{Collection, Item};
    use stac_api_backend::{Backend, MemoryBackend};
    use tower::ServiceExt;

    async fn test_config() -> Config {
        Config::from_toml("data/config.toml").await.unwrap()
    }

    #[tokio::test]
    async fn landing_page() {
        let api = super::api(MemoryBackend::new(), test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn collections() {
        let api = super::api(MemoryBackend::new(), test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn collection() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let api = super::api(backend, test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/an-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn items() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("a-collection", "a description"))
            .await
            .unwrap();
        let mut item = Item::new("an-item");
        item.collection = Some("a-collection".to_string());
        backend.add_item(item).await.unwrap();
        let api = super::api(backend, test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/a-collection/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn item() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("a-collection", "a description"))
            .await
            .unwrap();
        let item = Item::new("an-item").collection("a-collection");
        backend.add_item(item).await.unwrap();
        let api = super::api(backend, test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/a-collection/items/an-item")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{:?}",
            response.into_body().data().await
        );
    }

    #[tokio::test]
    async fn conformance() {
        let api = super::api(MemoryBackend::new(), test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/conformance")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{:?}",
            response.into_body().data().await
        );
    }
}
