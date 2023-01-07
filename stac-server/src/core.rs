use crate::{extract::Hrefs, Result, State};
use axum::{extract, Json};
use serde::Serialize;
use stac::{Catalog, Link, Links};
use stac_api::Backend;

#[derive(Debug, Serialize)]
pub struct LandingPage {
    #[serde(flatten)]
    pub catalog: Catalog,

    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}

impl LandingPage {
    async fn new(backend: impl Backend, mut catalog: Catalog, hrefs: Hrefs) -> Result<LandingPage> {
        catalog.set_link(Link::root(hrefs.root()));
        catalog.set_link(Link::self_(hrefs.root()));
        catalog.set_link(Link::new(hrefs.href("api"), "service-desc"));

        for collection in backend.collections().await? {
            catalog
                .links
                .push(Link::child(hrefs.collection(&collection)).title(collection.title));
        }

        Ok(LandingPage {
            catalog,
            conforms_to: vec!["https://api.stacspec.org/v1.0.0-rc.2/core".to_string()],
        })
    }
}

pub async fn landing_page<B: Backend>(
    extract::State(state): extract::State<State<B>>,
    hrefs: Hrefs,
) -> Json<LandingPage> {
    // TODO handle error pages
    Json(
        LandingPage::new(state.backend, state.catalog, hrefs)
            .await
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::LandingPage;
    use crate::extract::Hrefs;
    use stac::{Catalog, Collection, Links, Validate};
    use stac_api::{Backend, MemoryBackend};

    #[tokio::test]
    async fn new_landing_page() {
        let landing_page = LandingPage::new(
            MemoryBackend::new(),
            Catalog::new("an-id", "a description"),
            Hrefs::new(None),
        )
        .await
        .unwrap();

        let catalog = landing_page.catalog;

        let root_link = catalog.root_link().unwrap();
        assert_eq!(root_link.href, "/");

        let self_link = catalog.self_link().unwrap();
        assert_eq!(self_link.href, "/");

        let service_desc_link = catalog.link("service-desc").unwrap();
        assert_eq!(service_desc_link.href, "/api");

        catalog.validate().unwrap();

        let conforms_to = landing_page.conforms_to;
        assert!(conforms_to.contains(&"https://api.stacspec.org/v1.0.0-rc.2/core".to_string()));
    }

    #[tokio::test]
    async fn landing_page_with_collections() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let landing_page = LandingPage::new(
            backend,
            Catalog::new("an-id", "a description"),
            Hrefs::new(None),
        )
        .await
        .unwrap();
        let links: Vec<_> = landing_page.catalog.iter_child_links().collect();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].href, "/collections/an-id");
    }
}
