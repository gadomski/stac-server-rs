use crate::{Backend, Hrefs, Result};
use serde::Serialize;
use stac::{Catalog, Link, Links};

#[derive(Debug, Serialize)]
pub struct Root {
    #[serde(flatten)]
    pub catalog: Catalog,

    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}

impl Root {
    pub async fn new(backend: impl Backend, mut catalog: Catalog, hrefs: Hrefs) -> Result<Root> {
        let root = hrefs.root()?;
        catalog.set_link(Link::root(&root));
        catalog.set_link(Link::self_(root));
        catalog.set_link(Link::new(hrefs.href("api")?, "service-desc"));
        catalog.set_link(Link::new(hrefs.href("conformance")?, "conformance"));
        catalog.set_link(Link::new(hrefs.href("collections")?, "data"));

        for collection in backend.collections().await? {
            catalog
                .links
                .push(Link::child(hrefs.collection(&collection)?).title(collection.title));
        }

        Ok(Root {
            catalog,
            conforms_to: vec![
                "https://api.stacspec.org/v1.0.0-rc.2/core".to_string(),
                "https://api.stacspec.org/v1.0.0-rc.2/ogcapi-features".to_string(),
                "https://api.stacspec.org/v1.0.0-rc.2/collections".to_string(),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Root;
    use crate::{Backend, Hrefs, MemoryBackend};
    use stac::{Catalog, Collection, Links, Validate};

    #[tokio::test]
    async fn new_landing_page() {
        let landing_page = Root::new(
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

        let conformance_link = catalog.link("conformance").unwrap();
        assert_eq!(conformance_link.href, "/conformance");

        let collections_link = catalog.link("data").unwrap();
        assert_eq!(collections_link.href, "/collections");

        catalog.validate().unwrap();

        let conforms_to = landing_page.conforms_to;
        assert!(conforms_to.contains(&"https://api.stacspec.org/v1.0.0-rc.2/core".to_string()));
        assert!(conforms_to
            .contains(&"https://api.stacspec.org/v1.0.0-rc.2/ogcapi-features".to_string()));
        assert!(
            conforms_to.contains(&"https://api.stacspec.org/v1.0.0-rc.2/collections".to_string())
        );
    }

    #[tokio::test]
    async fn landing_page_with_collections() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let landing_page = Root::new(
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
