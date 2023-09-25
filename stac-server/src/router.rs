use crate::{Config, Error};
use aide::{
    axum::{routing::get, ApiRouter, IntoApiResponse},
    openapi::{Info, OpenApi},
};
use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::Html,
    Extension, Json, Router,
};
use stac_api::{GetItems, Root};
use stac_api_backend::{Api, Backend, Items};

/// Creates a new STAC API router.
///
/// # Examples
///
/// ```
/// use stac::Catalog;
/// use stac_api_backend::MemoryBackend;
/// use stac_server::Config;
///
/// let config = Config {
///     addr: "http://localhost:7822".to_string(),
///     features: true,
///     catalog: Catalog::new("an-id", "A description"),
/// };
/// let backend = MemoryBackend::new();
/// let api = stac_server::api(backend, config).unwrap();
/// ```
pub fn api<B: Backend + 'static>(backend: B, config: Config) -> crate::Result<Router>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    let root_url = format!("http://{}", config.addr); // TODO enable https
    let mut open_api = OpenApi {
        info: Info {
            description: Some(config.catalog.description.clone()),
            ..Info::default()
        },
        ..OpenApi::default()
    };
    let mut builder = Api::new(backend, config.catalog, &root_url)?;
    builder.features = config.features;
    let mut router = ApiRouter::new()
        .api_route("/", get(root))
        .api_route("/conformance", get(conformance));
    if builder.features {
        router = router
            .api_route("/collections", get(collections))
            .api_route("/collections/:collection_id", get(collection))
            .api_route("/collections/:collection_id/items", get(items))
            .api_route("/collections/:collection_id/items/:item_id", get(item));
    } else {
        router = router
            .api_route("/collections", get(not_implemented))
            .api_route("/collections/:collection_id", get(not_implemented))
            .api_route("/collections/:collection_id/items", get(not_implemented))
            .api_route(
                "/collections/:collection_id/items/:item_id",
                get(not_implemented),
            );
    }
    Ok(router
        .route("/api", get(service_desc))
        .route("/api.html", get(service_doc))
        .with_state(builder)
        .finish_api(&mut open_api)
        .layer(Extension(open_api)))
}

async fn root<B: Backend>(State(api): State<Api<B>>) -> Result<Json<Root>, (StatusCode, String)>
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    let root = api.root().await.map_err(internal_server_error)?;
    Ok(Json(root))
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

async fn service_doc<B: Backend>(State(api): State<Api<B>>) -> Html<String> {
    Html(format!("<!DOCTYPE html>
    <html>
      <head>
        <title>Redoc</title>
        <!-- needed for adaptive design -->
        <meta charset=\"utf-8\"/>
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
        <link href=\"https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700\" rel=\"stylesheet\">
    
        <!--
        Redoc doesn't change outer page styles
        -->
        <style>
          body {{
            margin: 0;
            padding: 0;
          }}
        </style>
      </head>
      <body>
        <redoc spec-url='{}'></redoc>
        <script src=\"https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js\"> </script>
      </body>
    </html>
    ", api.url_builder.service_desc()))
}

async fn conformance<B: Backend>(State(api): State<Api<B>>) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
{
    Json(api.conformance())
}

async fn collections<B: Backend>(State(api): State<Api<B>>) -> impl IntoApiResponse
where
    stac_api_backend::Error: From<<B as Backend>::Error>,
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
{
    match stac_api::Items::try_from(get_items)
        .map_err(Error::from)
        .and_then(|mut items| {
            let paging: B::Paging = serde_qs::from_str(&serde_qs::to_string(&std::mem::take(
                &mut items.additional_fields,
            ))?)?;
            Ok(Items { items, paging })
        }) {
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

async fn not_implemented() -> (StatusCode, String) {
    (StatusCode::NOT_IMPLEMENTED, "not implemented".to_string())
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use axum::{
        body::Body,
        http::{header::CONTENT_TYPE, Request, StatusCode},
    };
    use stac::{Catalog, Collection, Item};
    use stac_api_backend::{Backend, MemoryBackend};
    use tower::ServiceExt;

    fn test_config() -> Config {
        Config {
            addr: "http://localhost:7822".to_string(),
            features: true,
            catalog: Catalog::new("test-catalog", "A description"),
        }
    }

    #[tokio::test]
    async fn landing_page() {
        let api = super::api(MemoryBackend::new(), test_config()).unwrap();
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
        let api = super::api(MemoryBackend::new(), test_config()).unwrap();
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
        let api = super::api(MemoryBackend::new(), test_config()).unwrap();
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
        let api = super::api(backend, test_config()).unwrap();
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
        let api = super::api(backend, test_config()).unwrap();
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
        let api = super::api(backend, test_config()).unwrap();
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

    #[tokio::test]
    async fn no_features() {
        let mut config = test_config();
        config.features = false;
        let api = super::api(MemoryBackend::new(), config).unwrap();
        let response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/foo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/foo/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/collections/foo/items/bar")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let response = api
            .clone()
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
}
