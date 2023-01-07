use crate::State;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use stac::Collection;
use stac_api::Backend;
use std::convert::Infallible;

#[derive(Debug)]
pub struct Hrefs {
    addr: Option<String>,
}

impl Hrefs {
    pub fn new(addr: impl Into<Option<String>>) -> Hrefs {
        Hrefs { addr: addr.into() }
    }

    pub fn root(&self) -> String {
        self.href("")
    }

    pub fn href(&self, path: &str) -> String {
        if let Some(addr) = self.addr.as_ref() {
            // TODO allow protocol configuration
            format!("http://{}/{}", addr, path)
        } else {
            format!("/{}", path)
        }
    }

    pub fn collection(&self, collection: &Collection) -> String {
        // TODO DRY without making a bunch of strings
        if let Some(addr) = self.addr.as_ref() {
            format!("http://{}/collections/{}", addr, collection.id)
        } else {
            format!("/collections/{}", collection.id)
        }
    }
}

#[async_trait]
impl<B: Backend> FromRequestParts<State<B>> for Hrefs {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &State<B>) -> Result<Hrefs, Infallible> {
        Ok(Hrefs::new(state.addr.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::Hrefs;
    use crate::{Config, State};
    use axum::{extract::FromRequestParts, http::Request};
    use stac_api::MemoryBackend;

    #[tokio::test]
    async fn hrefs_use_addr() {
        let addr = "stac-server-rs.test:7822";
        let mut config = Config::from_toml("../data/config.toml").await.unwrap();
        config.addr = Some(addr.to_string());
        let state = State::new(MemoryBackend::new(), config);
        let (mut parts, _) = Request::builder().body(()).unwrap().into_parts();
        let hrefs = Hrefs::from_request_parts(&mut parts, &state).await.unwrap();
        assert_eq!(hrefs.root(), "http://stac-server-rs.test:7822/");
    }
}
