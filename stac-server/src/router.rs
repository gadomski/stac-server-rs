use crate::Config;
use aide::{
    axum::{routing::get, ApiRouter, IntoApiResponse},
    openapi::OpenApi,
};
use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    Extension, Json,
};
use stac_api::{GetItems, Items};
use stac_api_backend::{Api, Backend, Page};

/// Creates a new STAC API router.
///
/// # Examples
///
/// ```
/// use stac_server::{Config, CatalogConfig};
/// use stac_api_backend::MemoryBackend;
///
/// let config = Config {
///     addr: "http://localhost:7822".to_string(),
///     catalog: CatalogConfig {
///         id: "an-id".to_string(),
///         description: "a description".to_string(),
///     },
/// };
/// let backend = MemoryBackend::new();
/// let api = stac_server::api(backend, config).unwrap();
/// ```
pub fn api<B: Backend + 'static>(backend: B, config: Config) -> crate::Result<ApiRouter>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    let root_url = format!("http://{}", config.addr); // TODO enable https
    let catalog = config.catalog.into_catalog();
    let builder = Api::new(backend, catalog, &root_url)?;
    Ok(ApiRouter::new()
        .api_route("/", get(root))
        .api_route("/conformance", get(conformance))
        .api_route("/collections", get(collections))
        .api_route("/collections/:collection_id", get(collection))
        .api_route("/collections/:collection_id/items", get(items))
        .api_route("/collections/:collection_id/items/:item_id", get(item))
        .route("/api", get(service_desc))
        .with_state(builder))
}

async fn root<B: Backend>(State(api): State<Api<B>>) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    api.root().await.map(Json).map_err(internal_server_error)
}

async fn service_desc(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    let mut headers = HeaderMap::new();
    let _ = headers.insert(
        CONTENT_TYPE,
        "application/vnd.oai.openapi+json;version=3.1"
            .parse()
            .unwrap(),
    );
    (headers, Json(api))
}

async fn conformance<B: Backend>(State(api): State<Api<B>>) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    Json(api.conformance())
}

async fn collections<B: Backend>(State(api): State<Api<B>>) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    api.collections()
        .await
        .map(Json)
        .map_err(internal_server_error)
}

async fn collection<B: Backend>(
    State(api): State<Api<B>>,
    Path(collection_id): Path<String>,
) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    if let Some(collection) = api
        .collection(&collection_id)
        .await
        .map_err(internal_server_error)?
    {
        return Ok(Json(collection));
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            format!("no collection with id={}", collection_id),
        ));
    }
}

async fn items<B: Backend>(
    State(api): State<Api<B>>,
    Path(collection_id): Path<String>,
    Query(get_items): Query<GetItems>,
) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    match Items::try_from(get_items) {
        Ok(items) => {
            if let Some(items) = api
                .items(&collection_id, items)
                .await
                .map_err(internal_server_error)?
            {
                let mut headers = HeaderMap::new();
                let _ = headers.insert(CONTENT_TYPE, "application/geo+json".parse().unwrap());
                return Ok((headers, Json(items)));
            } else {
                return Err((
                    StatusCode::NOT_FOUND,
                    format!("no collection with id={}", collection_id),
                ));
            }
        }
        Err(err) => Err((StatusCode::BAD_REQUEST, format!("invalid query: {}", err))),
    }
}

async fn item<B: Backend>(
    State(api): State<Api<B>>,
    Path((collection_id, item_id)): Path<(String, String)>,
) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
    stac_api_backend::Error: From<<<B as Backend>::Page as Page>::Error>,
{
    if let Some(item) = api
        .item(&collection_id, &item_id)
        .await
        .map_err(internal_server_error)?
    {
        let mut headers = HeaderMap::new();
        let _ = headers.insert(CONTENT_TYPE, "application/geo+json".parse().unwrap());
        return Ok((headers, Json(item)));
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            format!(
                "no item with id={} in collection={}",
                item_id, collection_id
            ),
        ));
    }
}

fn internal_server_error(err: stac_api_backend::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("internal server error: {}", err),
    )
}

#[cfg(test)]
mod tests {
    use crate::{CatalogConfig, Config};
    use axum::{
        body::Body,
        http::{header::CONTENT_TYPE, Request, StatusCode},
    };
    use stac::{Collection, Item};
    use stac_api_backend::{Backend, MemoryBackend};
    use tower::ServiceExt;

    async fn test_config() -> Config {
        Config {
            addr: "http://localhost:7822".to_string(),
            catalog: CatalogConfig {
                id: "test-catalog".to_string(),
                description: "A description".to_string(),
            },
        }
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
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn collection() {
        let mut backend = MemoryBackend::new();
        let _ = backend
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
        let _ = backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let api = super::api(backend, test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/an-id/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/geo+json"
        );
    }

    #[tokio::test]
    async fn item() {
        let mut backend = MemoryBackend::new();
        let _ = backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        backend
            .add_items(vec![Item::new("item-id").collection("an-id")])
            .await
            .unwrap();
        let api = super::api(backend, test_config().await).unwrap();
        let response = api
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/an-id/items/item-id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK,);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/geo+json"
        );
    }
}
