use stac_api::ItemCollection;
use std::error::Error;
use url::Url;

/// A trait defining behavior for pages.
///
/// Pages are used as the response type from item search and the items endpoint.
pub trait Page {
    /// The error type for this page.
    type Error: Error;

    /// Converts this page into an item collection.
    ///
    /// # Examples
    ///
    /// TODO
    fn into_item_collection(self, url: Url) -> Result<ItemCollection, Self::Error>;
}
