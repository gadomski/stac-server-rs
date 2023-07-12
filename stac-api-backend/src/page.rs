use stac_api::ItemCollection;

/// A page of items.
#[derive(Debug)]
pub struct Page<P> {
    /// The items.
    pub item_collection: ItemCollection,

    /// The paging data for the next link.
    pub next: Option<P>,

    /// The paging data for the prev link.
    pub prev: Option<P>,
}
