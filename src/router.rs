use crate::{
    handler::{collection, collections, landing_page},
    ApiState, Backend, Config,
};
use axum::{routing::get, Router};

pub fn api<B: Backend>(config: Config, backend: B) -> Router {
    let mut router = Router::new().route("/", get(landing_page));
    if config.ogc_api_features {
        router = router
            .route("/collections", get(collections))
            .route("/collections/:id", get(collection));
    }
    router.with_state(ApiState { config, backend })
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::{Backend, Simple},
        config::{CatalogConfig, Config},
        handler::{CollectionsPage, LandingPage},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use serde::de::DeserializeOwned;
    use stac::{media_type, Collection, Links};
    use std::net::{SocketAddr, TcpListener};
    use tower::util::ServiceExt;

    fn test_config() -> Config {
        Config {
            browseable: false,
            ogc_api_features: false,
            catalog: CatalogConfig {
                id: "test-id".to_string(),
                description: "unit tests".to_string(),
                title: Some("the title".to_string()),
            },
        }
    }

    async fn get<D: DeserializeOwned>(api: Router, uri: &str) -> D {
        let response = api
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "{:?}",
            hyper::body::to_bytes(response.into_body()).await.unwrap()
        );
        serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await.unwrap()).unwrap()
    }

    #[tokio::test]
    async fn landing_page() {
        let api = crate::api(test_config(), Simple::new());
        let landing_page: LandingPage = get(api, "/").await;
        // catalog.validate().unwrap();  // depends on https://github.com/gadomski/stac-rs/issues/27
        assert!(landing_page
            .conforms_to
            .contains(&"https://api.stacspec.org/v1.0.0-rc.2/core".to_string()));
        let catalog = landing_page.catalog;
        assert_eq!(catalog.id, "test-id");
        assert_eq!(catalog.description, "unit tests");
        assert_eq!(catalog.title.as_ref().unwrap(), "the title");
        let root = catalog.root_link().unwrap();
        assert_eq!(root.r#type.as_ref().unwrap(), media_type::JSON);
        let self_ = catalog.self_link().unwrap();
        assert_eq!(self_.r#type.as_ref().unwrap(), media_type::JSON);
        assert_eq!(root.href, self_.href);
        // TODO check service-desc
        // TODO check service-doc
        assert_eq!(catalog.iter_child_links().count(), 0);
        assert_eq!(catalog.iter_item_links().count(), 0);
    }

    #[tokio::test]
    async fn landing_page_with_child() {
        let mut collection = Collection::new("an-id");
        collection.title = Some("The title".to_string());
        let mut simple = Simple::new();
        simple.add_collection(collection).await.unwrap();
        let api = crate::api(test_config(), simple);
        let landing_page: LandingPage = get(api, "/").await;
        let catalog = landing_page.catalog;
        let child_links: Vec<_> = catalog.iter_child_links().collect();
        assert_eq!(child_links.len(), 1);
        let link = child_links[0];
        assert_eq!(link.href, "/collections/an-id");
        assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
        assert_eq!(link.title.as_ref().unwrap(), "The title");
    }

    #[tokio::test]
    async fn landing_page_with_browsable() {
        let mut config = test_config();
        config.browseable = true;
        let api = crate::api(config, Simple::new());
        let landing_page: LandingPage = get(api, "/").await;
        assert!(landing_page
            .conforms_to
            .contains(&"https://api.stacspec.org/v1.0.0-rc.2/browseable".to_string()));
    }

    #[tokio::test]
    async fn ogc_api_features() {
        let mut config = test_config();
        config.ogc_api_features = true;
        let mut backend = Simple::new();
        backend
            .add_collection(Collection::new("test-collection"))
            .await
            .unwrap();
        let api = crate::api(config, backend);
        let landing_page: LandingPage = get(api.clone(), "/").await;

        let conforms_to = landing_page.conforms_to;
        assert!(conforms_to
            .contains(&"https://api.stacspec.org/v1.0.0-rc.2/ogcapi-features".to_string()));
        assert!(conforms_to
            .contains(&"http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core".to_string()));
        assert!(conforms_to.contains(
            &"http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson".to_string()
        ));

        let catalog = landing_page.catalog;
        let conformance_link = catalog.link("conformance").unwrap();
        assert_eq!(conformance_link.href, "/conformance");
        let collections_link = catalog.link("data").unwrap();
        assert_eq!(collections_link.href, "/collections");

        let collections_page: CollectionsPage = get(api.clone(), &collections_link.href).await;
        let root_link = collections_page.root_link().unwrap();
        assert_eq!(root_link.href, "/");
        let self_link = collections_page.self_link().unwrap();
        assert_eq!(self_link.href, "/collections");
        assert_eq!(collections_page.collections.len(), 1);

        let collection: Collection = get(api, "/collections/test-collection").await;
        let root_link = collection.root_link().unwrap();
        assert_eq!(root_link.href, "/");
        let parent_link = collection.link("parent").unwrap(); // TODO add parent link to links
        assert_eq!(parent_link.href, "/");
        let self_link = collection.self_link().unwrap();
        assert_eq!(self_link.href, "/collections/test-collection");
    }

    #[tokio::test]
    async fn landing_page_with_server() {
        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(crate::api(test_config(), Simple::new()).into_make_service())
                .await
                .unwrap();
        });

        let client = hyper::Client::new();
        let response = client
            .request(
                Request::builder()
                    .uri(format!("http://{}/", addr))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let landing_page: LandingPage =
            serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await.unwrap())
                .unwrap();
        let root = landing_page
            .catalog
            .links
            .iter()
            .find(|link| link.is_root())
            .unwrap();
        assert_eq!(root.href, format!("http://{}/", addr));
    }
}
