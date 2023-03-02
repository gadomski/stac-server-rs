use stac_api::ItemCollection;

/// A paginated [ItemCollection].
///
///
#[derive(Debug)]
pub struct PaginatedItemCollection<P> {
    /// The [ItemCollection].
    pub item_collection: ItemCollection,

    /// The pagination for the next link.
    pub next: Option<P>,

    /// The pagination for the prev link.
    pub prev: Option<P>,
}
