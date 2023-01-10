use serde::{Deserialize, Serialize};
use stac::{Catalog, Collection, Link};
use url::Url;

/// The root landing page of a STAC API.
///
/// In a STAC API, the root endpoint (Landing Page) has the following characteristics:
///
///   - The returned JSON is a [STAC
///     Catalog](../stac-spec/catalog-spec/catalog-spec.md), and provides any number
///     of 'child' links to navigate to additional
///     [Catalog](../stac-spec/catalog-spec/catalog-spec.md),
///     [Collection](../stac-spec/collection-spec/README.md), and
///     [Item](../stac-spec/item-spec/README.md) objects.
///   - The `links` attribute is part of a STAC Catalog, and provides a list of
///     relations to API endpoints. Some of these endpoints can exist on any path
///     (e.g., sub-catalogs) and some have a specified path (e.g., `/search`), so
///     the client must inspect the `rel` (relationship) to understand what
///     capabilities are offered at each location.
///   - The `conformsTo` section provides the capabilities of this service. This
///     is the field that indicates to clients that this is a STAC API and how to
///     access conformance classes, including this one. The relevant conformance
///     URIs are listed in each part of the API specification. If a conformance
///     URI is listed then the service must implement all of the required
///     capabilities.
///
/// Note the `conformsTo` array follows the same structure of the OGC API -
/// Features [declaration of conformance
/// classes](http://docs.opengeospatial.org/is/17-069r3/17-069r3.html#_declaration_of_conformance_classes),
/// except it is part of the landing page instead of in the JSON response from
/// the `/conformance` endpoint. This is different from how the OGC API
/// advertises conformance, as STAC feels it is important for clients to
/// understand conformance from a single request to the landing page.
/// Implementers who implement the *OGC API - Features* and/or *STAC API -
/// Features* conformance classes must also implement the `/conformance`
/// endpoint.

#[derive(Debug, Serialize, Deserialize)]
pub struct Root {
    #[serde(flatten)]
    pub catalog: Catalog,

    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}

/// Object containing an array of Collection objects in the Catalog, and Link relations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Collections {
    /// The [Collection] objects in the [Catalog].
    pub collections: Vec<Collection>,

    /// The [stac::Link] relations.
    pub links: Vec<Link>,
}

/// Build links to endpoings in a STAC API.
#[derive(Debug)]
pub struct LinkBuilder(Option<Url>);

impl LinkBuilder {
    /// Creates a new link builder with the given root.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// # use url::Url;
    /// let root = Url::parse("http://stac-api-rs.test/api/v1").unwrap();
    /// let link_builder = LinkBuilder::new(root);
    /// ```
    pub fn new(url: impl Into<Option<Url>>) -> LinkBuilder {
        LinkBuilder(url.into())
    }

    /// Returns a root link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// # use url::Url;
    /// let root = Url::parse("http://stac-api-rs.test/api/v1").unwrap();
    /// let link_builder = LinkBuilder::new(root);
    /// let root = link_builder.root();
    /// assert_eq!(root.rel, "root");
    /// assert_eq!(root.href, "http://stac-api-rs.test/api/v1");
    /// ```
    pub fn root(&self) -> Link {
        if let Some(root) = &self.0 {
            Link::root(root.as_str())
        } else {
            Link::root("/")
        }
    }
}
