use crate::{Config, Result, State};
use axum::{
    extract::{Path, State as AxumState},
    routing::get,
    Json, Router,
};
use stac::Collection;
use stac_api::{Collections, LinkBuilder, Root};
use stac_backend::Backend;

/// Returns the STAC API router.
///
/// # Examples
///
/// ```
/// use stac_backend::MemoryBackend;
/// use stac_server::Config;
///
/// # tokio_test::block_on(async {
/// let config = Config::from_toml("data/config.toml").await.unwrap();
/// let api = stac_server::api(MemoryBackend::new(), config).unwrap();
/// # })
/// ```
pub fn api<B: Backend + 'static>(backend: B, config: Config) -> Result<Router> {
    let state = State::new(backend, config)?;
    Ok(Router::new()
        .route("/", get(root))
        .route("/collections", get(collections))
        .route("/collections/:id", get(collection))
        .with_state(state))
}

async fn root<B: Backend>(
    AxumState(state): AxumState<State<B>>,
    link_builder: LinkBuilder,
) -> Json<Root> {
    // TODO handle error pages
    Json(
        state
            .backend
            .root_endpoint(link_builder, state.catalog)
            .await
            .unwrap(),
    )
}

async fn collections<B: Backend>(
    AxumState(state): AxumState<State<B>>,
    link_builder: LinkBuilder,
) -> Json<Collections> {
    // TODO handle error pages
    Json(
        state
            .backend
            .collections_endpoint(link_builder)
            .await
            .unwrap(),
    )
}

pub async fn collection<B: Backend>(
    AxumState(state): AxumState<State<B>>,
    link_builder: LinkBuilder,
    Path(id): Path<String>,
) -> Json<Collection> {
    // TODO handle error pages
    Json(
        state
            .backend
            .collection_endpoint(link_builder, &id)
            .await
            .unwrap()
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use stac::Collection;
    use stac_backend::{Backend, MemoryBackend};
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
}
