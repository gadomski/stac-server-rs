use crate::{Config, State};
use axum::{routing::get, Router};
use stac_backend::Backend;

/// Returns the STAC API router.
///
/// # Examples
///
/// ```
/// use stac_server::Config;
/// use stac_backend::MemoryBackend;
///
/// # tokio_test::block_on(async {
/// let config = Config::from_toml("data/config.toml").await.unwrap();
/// let api = stac_server::api(MemoryBackend::new(), config);
/// # })
/// ```
pub fn api<B: Backend + 'static>(backend: B, config: Config) -> Router {
    let state = State::new(backend, config);
    Router::new()
        .route("/", get(crate::core::landing_page))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use crate::Config;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use stac_backend::MemoryBackend;
    use tower::ServiceExt;

    async fn test_config() -> Config {
        Config::from_toml("data/config.toml").await.unwrap()
    }

    #[tokio::test]
    async fn landing_page() {
        let api = super::api(MemoryBackend::new(), test_config().await);
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
}
