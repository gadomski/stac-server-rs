use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::{Catalog, Collection, Link};
use thiserror::Error;
use url::Url;

const ITEM_COLLECTION_TYPE: &str = "FeatureCollection";

/// Crate-specific error type.
#[derive(Debug, Error)]
pub enum Error {
    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),
}

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

/// The return value of the /items and /search endpoints.
///
/// This can be a [stac::ItemCollection], but if the [fields
/// extension](https://github.com/stac-api-extensions/fields) is used, it might
/// not be.
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemCollection {
    /// Always "FeatureCollection" to provide compatibility with GeoJSON.
    pub r#type: String,

    /// A possibly-empty array of Item objects.
    pub features: Vec<Item>,

    /// An array of Links related to this ItemCollection.
    pub links: Vec<Link>,

    /// The number of Items that meet the selection parameters, possibly estimated.
    #[serde(skip_serializing_if = "Option::is_none", rename = "numberMatched")]
    pub number_matched: Option<u64>,

    /// The number of Items in the features array.
    #[serde(skip_serializing_if = "Option::is_none", rename = "numberReturned")]
    pub number_returned: Option<u64>,
}

/// A STAC item, as returned by a STAC API implementation.
///
/// Just an arbitrary dictionary, since it could have required fields removed.
#[derive(Debug, Serialize, Deserialize)]
pub struct Item(pub Map<String, Value>);

/// Build links to endpoints in a STAC API.
#[derive(Debug)]
pub struct LinkBuilder(Url);

impl ItemCollection {
    /// Creates a new [ItemCollection] from an items.
    ///
    /// # Examples
    ///
    /// TODO
    pub fn new(items: Vec<Item>) -> Result<ItemCollection, Error> {
        let number_returned = items.len();
        Ok(ItemCollection {
            r#type: ITEM_COLLECTION_TYPE.to_string(),
            features: items,
            links: Vec::new(),
            number_matched: None,
            number_returned: Some(number_returned.try_into()?),
        })
    }
}

impl TryFrom<stac::Item> for Item {
    type Error = serde_json::Error;

    fn try_from(item: stac::Item) -> Result<Item, serde_json::Error> {
        match serde_json::to_value(item)? {
            Value::Object(object) => Ok(Item(object)),
            _ => panic!("a STAC item shouldn't be able to deserialize to anything but an object"),
        }
    }
}

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
    pub fn new(url: Url) -> LinkBuilder {
        LinkBuilder(url)
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
        Link::root(self.0.as_str())
    }
}
