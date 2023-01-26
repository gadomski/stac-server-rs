use crate::Result;
use serde::Serialize;
use serde_json::{Map, Value};
use stac::Link;

#[derive(Debug)]
pub struct PaginationLinks<Q: Serialize> {
    pub next: Option<UnresolvedLink<Q>>,
    pub prev: Option<UnresolvedLink<Q>>,
}

#[derive(Debug)]
pub struct UnresolvedLink<Q: Serialize> {
    pub query: Q,
    pub title: Option<String>,
    pub additional_fields: Map<String, Value>,
}

impl<Q: Serialize> PaginationLinks<Q> {
    pub fn new(next: impl Into<Option<Q>>, prev: impl Into<Option<Q>>) -> PaginationLinks<Q> {
        PaginationLinks {
            next: next.into().map(UnresolvedLink::new),
            prev: prev.into().map(UnresolvedLink::new),
        }
    }
}

impl<Q: Serialize> UnresolvedLink<Q> {
    pub fn new(query: Q) -> UnresolvedLink<Q> {
        UnresolvedLink {
            query,
            title: None,
            additional_fields: Map::new(),
        }
    }

    pub fn resolve(mut self, mut link: Link) -> Result<Link> {
        // TODO handle POST
        link.set_query(self.query)?;
        link.title = self.title;
        link.additional_fields.append(&mut self.additional_fields);
        Ok(link)
    }
}
