use crate::State;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use stac_api::LinkBuilder;
use stac_backend::Backend;
use std::convert::Infallible;

#[async_trait]
impl<B: Backend> FromRequestParts<State<B>> for LinkBuilder {
    type Rejection = Infallible;

    async fn from_request_parts(
        _: &mut Parts,
        state: &State<B>,
    ) -> Result<LinkBuilder, Infallible> {
        Ok(LinkBuilder::new(state.root.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Config, State};
    use axum::{extract::FromRequestParts, http::Request};
    use stac_api::LinkBuilder;
    use stac_backend::MemoryBackend;

    #[tokio::test]
    async fn hrefs_use_addr() {
        let addr = "stac-server-rs.test:7822";
        let mut config = Config::from_toml("../data/config.toml").await.unwrap();
        config.addr = Some(addr.to_string());
        let state = State::new(MemoryBackend::new(), config).unwrap();
        let (mut parts, _) = Request::builder().body(()).unwrap().into_parts();
        let link_builder = LinkBuilder::from_request_parts(&mut parts, &state)
            .await
            .unwrap();
        assert_eq!(link_builder.root().href, "http://stac-server-rs.test:7822/");
    }
}
