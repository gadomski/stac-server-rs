use serde::Serialize;
use std::fmt::Debug;

/// A query for items.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Items<P>
where
    P: Debug + Clone + Serialize + Default,
{
    #[serde(flatten)]
    /// The items query.
    pub items: stac_api::Items,

    #[serde(flatten)]
    /// The backend-specific paging structure
    pub paging: P,
}

/// A get query for items.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GetItems<P>
where
    P: Debug + Clone + Serialize + Default,
{
    #[serde(flatten)]
    /// The items query.
    pub get_items: stac_api::GetItems,

    #[serde(flatten)]
    /// The backend-specific paging structure
    pub paging: P,
}
