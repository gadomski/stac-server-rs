use crate::{Backend, Error, Result};
use stac::{Catalog, Collection, Item, Links};
use stac_api::{Collections, Conformance, ItemCollection, LinkBuilder, Root};

// TODO move these to stac-api
const CONFORMANCE_CLASSES: [&str; 5] = [
    "https://api.stacspec.org/v1.0.0-rc.2/core",
    "https://api.stacspec.org/v1.0.0-rc.2/ogcapi-feautres",
    "https://api.stacspec.org/v1.0.0-rc.2/collections",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson",
];

/// A structure for generating STAC API endpoints.
#[derive(Clone, Debug)]
pub struct Endpoints<B: Backend> {
    backend: B,
    catalog: Catalog,
    link_builder: LinkBuilder,
}

impl<B: Backend> Endpoints<B>
where
    Error: From<<B as Backend>::Error>,
{
    /// Creates a new endpoint generator with the given backend, catalog, and link builder.
    ///
    /// The catalog is used as the root endpoint.
    ///
    /// # Examples
    ///
    /// TODO
    pub fn new(backend: B, catalog: Catalog, link_builder: LinkBuilder) -> Endpoints<B> {
        Endpoints {
            backend,
            catalog,
            link_builder,
        }
    }

    /// Returns the root endpoint, as defined by
    /// <https://github.com/radiantearth/stac-api-spec/tree/main/core#endpoints>.
    ///
    /// # Examples
    ///
    /// TODO
    pub async fn root(&self) -> Result<Root> {
        let mut catalog = self.catalog.clone();
        // TODO check to make sure these links don't already exist
        catalog.links = vec![
            self.link_builder.root(),
            self.link_builder.root_to_self(),
            self.link_builder.service_desc(),
            self.link_builder.conformance(),
            self.link_builder.collections(),
        ];
        for collection in self.backend.collections().await? {
            catalog
                .links
                .push(self.link_builder.root_to_collection(&collection.id)?);
        }
        Ok(Root {
            catalog,
            conformance: self.conformance(),
        })
    }

    /// Returns the conformance endpoint.
    pub fn conformance(&self) -> Conformance {
        Conformance {
            conforms_to: CONFORMANCE_CLASSES.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Returns the collections endpoint.
    pub async fn collections(&self) -> Result<Collections> {
        let links = vec![
            self.link_builder.root(),
            self.link_builder.collections_to_self(),
        ];
        Ok(Collections {
            collections: vec![],
            links,
            additional_fields: Default::default(),
        })
    }

    /// Returns the endpoint for a single collection.
    pub async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        if let Some(mut collection) = self.backend.collection(id).await? {
            // TODO make sure we're not repeating links.
            collection.links.extend([
                self.link_builder.root(),
                self.link_builder.collection_to_parent(),
                self.link_builder.collection_to_self(id)?,
                self.link_builder.collection_to_items(id)?,
            ]);
            Ok(Some(collection))
        } else {
            Ok(None)
        }
    }

    /// Returns items.
    pub async fn items(
        &self,
        id: &str,
        pagination: Option<B::Pagination>,
    ) -> Result<Option<ItemCollection>> {
        if let Some(paginated_item_collection) = self.backend.items(id, pagination).await? {
            let mut item_collection = paginated_item_collection.item_collection;
            if let Some(next) = paginated_item_collection.next {
                let link = self.link_builder.next_items(id, next)?;
                item_collection.set_link(link);
            }
            if let Some(prev) = paginated_item_collection.prev {
                let link = self.link_builder.prev_items(id, prev)?;
                item_collection.set_link(link);
            }
            Ok(Some(item_collection))
        } else {
            Ok(None)
        }
    }

    /// Returns a single item.
    pub async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>> {
        self.backend
            .item(collection_id, item_id)
            .await
            .map_err(Error::from)
    }
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use super::Endpoints;
    use crate::{
        memory::{MemoryBackend, Pagination},
        Backend,
    };
    use stac::{Catalog, Collection, Item, Links, Validate};

    fn builder() -> Endpoints<MemoryBackend> {
        Endpoints::new(
            MemoryBackend::new(),
            Catalog::new("test-catalog", "A catalog for testing"),
            "http://stac-api-backend-rs.test".parse().unwrap(),
        )
    }

    fn assert_conformance_classes(conforms_to: &[String]) {
        for conformance_class in [
            "https://api.stacspec.org/v1.0.0-rc.2/ogcapi-feautres",
            "https://api.stacspec.org/v1.0.0-rc.2/collections",
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core",
            "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson",
        ] {
            assert!(
                conforms_to.contains(&conformance_class.to_string()),
                "missing conformance class: {}",
                conformance_class
            );
        }
    }

    #[tokio::test]
    async fn core() {
        let root = builder().root().await.unwrap();
        assert!(root
            .conformance
            .conforms_to
            .contains(&"https://api.stacspec.org/v1.0.0-rc.2/core".to_string()));

        let catalog = root.catalog;
        catalog.clone().validate().unwrap();
        assert_eq!(
            catalog.root_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/"
        );
        assert_eq!(
            catalog.self_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/"
        );
        assert_eq!(
            catalog.link("service-desc").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/api"
        );
    }

    #[tokio::test]
    async fn root_with_collections() {
        let mut builder = builder();
        builder
            .backend
            .add_collection(Collection::new("collection-a", "The first collection"))
            .await
            .unwrap();
        builder
            .backend
            .add_collection(Collection::new("collection-b", "The first collection"))
            .await
            .unwrap();
        let root = builder.root().await.unwrap();
        let child_links: Vec<_> = root.catalog.iter_child_links().collect();
        assert_eq!(child_links.len(), 2);
    }

    #[tokio::test]
    async fn features() {
        let mut builder = builder();
        let root = builder.clone().root().await.unwrap();
        assert_conformance_classes(&root.conformance.conforms_to);
        let catalog = root.catalog;
        assert_eq!(
            catalog.link("conformance").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/conformance"
        );
        assert_eq!(
            catalog.link("data").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections"
        );

        builder
            .backend
            .add_collection(Collection::new("collection-a", "The first collection"))
            .await
            .unwrap();
        builder
            .backend
            .add_collection(Collection::new("collection-b", "The first collection"))
            .await
            .unwrap();
        let collections = builder.clone().collections().await.unwrap();
        assert_eq!(
            collections.root_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/"
        );
        assert_eq!(
            collections.self_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections"
        );

        let collection = builder
            .clone()
            .collection("collection-a")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            collection.root_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/"
        );
        assert_eq!(
            collection.parent_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/"
        );
        assert_eq!(
            collection.self_link().as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections/collection-a"
        );
        assert_eq!(
            collection.link("items").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections/collection-a/items"
        );

        let item_collection = builder
            .clone()
            .items("collection-a", None)
            .await
            .unwrap()
            .unwrap();
        assert!(item_collection.items.is_empty());

        builder
            .backend
            .add_item(Item::new("item-id").collection("collection-a"))
            .await
            .unwrap();
        let item_collection = builder
            .clone()
            .items("collection-a", None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(item_collection.items.len(), 1);
        let _ = builder
            .clone()
            .item("collection-a", "item-id")
            .await
            .unwrap()
            .unwrap();
        assert!(builder
            .item("collection-b", "item-id")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn pagination() {
        let mut builder = builder();
        builder
            .backend
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        for i in 0..10 {
            builder
                .backend
                .add_item(Item::new(format!("item-{}", i)).collection("an-id"))
                .await
                .unwrap();
        }
        let items = builder
            .clone()
            .items("an-id", Some(Pagination { skip: 0, take: 2 }))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(items.items.len(), 2);
        assert_eq!(
            items.link("next").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections/an-id/items?skip=2&take=2"
        );
        let items = builder
            .clone()
            .items("an-id", Some(Pagination { skip: 2, take: 2 }))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(items.items.len(), 2);
        assert_eq!(
            items.link("next").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections/an-id/items?skip=4&take=2"
        );
        assert_eq!(
            items.link("prev").as_ref().unwrap().href,
            "http://stac-api-backend-rs.test/collections/an-id/items?skip=0&take=2"
        );

        let conformance = builder.conformance();
        assert_conformance_classes(&conformance.conforms_to);
    }
}
