use crate::{Api, Backend, Error, Result};
use stac::Link;
use stac_api::Root;

impl<B> Api<B>
where
    B: Backend,
    Error: From<<B as Backend>::Error>,
{
    /// Returns the [root endpoint](https://github.com/radiantearth/stac-api-spec/tree/main/core#endpoints).
    pub async fn root(&self) -> Result<Root> {
        let mut catalog = self.catalog.clone();
        catalog.links.extend([
            Link::root(self.url_builder.root()),
            Link::self_(self.url_builder.root()),
            Link::new(self.url_builder.service_desc(), "service-desc")
                .r#type(self.service_desc_media_type.clone()),
            Link::new(
                format!("{}.html", self.url_builder.service_desc()),
                "service-doc",
            )
            .r#type("text/html".to_string()),
        ]);
        if self.features {
            catalog.links.push(
                Link::new(self.url_builder.collections(), "data")
                    .json()
                    .title("Collections".to_string()),
            );
            catalog.links.push(
                Link::new(self.url_builder.conformance(), "conformance")
                    .json()
                    .title("Conformance".to_string()),
            );
        }
        for collection in self.backend.collections().await? {
            catalog.links.push(
                Link::child(self.url_builder.collection(&collection.id)?).title(collection.title),
            )
        }
        Ok(Root {
            catalog,
            conformance: self.conformance(),
        })
    }
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use super::super::tests;
    use crate::{assert_link, Backend, DEFAULT_SERVICE_DESC_MEDIA_TYPE};
    use stac::{Collection, Links};
    use stac_api::{COLLECTIONS_URI, CORE_URI, FEATURES_URI, GEOJSON_URI, OGC_API_FEATURES_URI};
    use stac_validate::Validate;

    #[tokio::test]
    async fn default_conformance_classes() {
        let root = tests::api().root().await.unwrap();
        for uri in [
            CORE_URI,
            FEATURES_URI,
            COLLECTIONS_URI,
            OGC_API_FEATURES_URI,
            GEOJSON_URI,
        ] {
            assert!(
                root.conformance.conforms_to.contains(&uri.to_string()),
                "does not conform to {}",
                uri
            );
        }
    }

    #[tokio::test]
    async fn is_valid() {
        let root = tests::api().root().await.unwrap();
        root.catalog.validate().unwrap();
    }

    #[tokio::test]
    async fn links() {
        let root = tests::api().root().await.unwrap();
        assert_link!(
            root.catalog,
            "root",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            root.catalog,
            "self",
            "http://stac-api-backend.test/",
            "application/json"
        );
        assert_link!(
            root.catalog,
            "service-desc",
            "http://stac-api-backend.test/api",
            DEFAULT_SERVICE_DESC_MEDIA_TYPE
        );
        assert_link!(
            root.catalog,
            "service-doc",
            "http://stac-api-backend.test/api.html",
            "text/html"
        );
    }

    #[tokio::test]
    async fn child() {
        let mut api = tests::api();
        let _ = api
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let root = api.root().await.unwrap();
        assert_eq!(root.catalog.iter_child_links().count(), 1);
        assert_link!(
            root.catalog,
            "child",
            "http://stac-api-backend.test/collections/an-id",
            "application/json"
        );
    }
}
