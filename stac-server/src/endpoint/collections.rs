use crate::{extract::Hrefs, Result, State};
use axum::{extract, Json};
use serde::Serialize;
use stac::{Collection, Link, Links};
use stac_api::Backend;

#[derive(Debug, Serialize)]
pub struct Collections {
    pub collections: Vec<Collection>,
    pub links: Vec<Link>,
}

impl Collections {
    async fn new(backend: impl Backend, hrefs: Hrefs) -> Result<Collections> {
        let collections = backend.collections().await.unwrap();
        let links = vec![
            Link::root(hrefs.root()),
            Link::self_(hrefs.href("collections")),
        ];
        Ok(Collections { collections, links })
    }
}

impl Links for Collections {
    fn links(&self) -> &[Link] {
        &self.links
    }

    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

pub async fn collections<B: Backend>(
    extract::State(state): extract::State<State<B>>,
    hrefs: Hrefs,
) -> Json<Collections> {
    // TODO handle error pages
    Json(Collections::new(state.backend, hrefs).await.unwrap())
}

#[cfg(test)]
mod tests {
    use super::Collections;
    use crate::extract::Hrefs;
    use stac::Links;
    use stac_api::MemoryBackend;

    #[tokio::test]
    async fn collections() {
        let collections = Collections::new(MemoryBackend::new(), Hrefs::new(None))
            .await
            .unwrap();

        let root_link = collections.root_link().unwrap();
        assert_eq!(root_link.href, "/");
        let self_link = collections.self_link().unwrap();
        assert_eq!(self_link.href, "/collections");

        assert!(collections.collections.is_empty());
    }
}
