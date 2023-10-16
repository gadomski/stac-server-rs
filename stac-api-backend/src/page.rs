use crate::Result;
use http::Method;
use serde::Serialize;
use stac::Link;
use stac_api::ItemCollection;
use url::Url;

/// A page of items.
#[derive(Debug)]
pub struct Page<P: Serialize> {
    /// The items.
    pub item_collection: ItemCollection,

    /// The paging data for the next link.
    pub next: Option<P>,

    /// The paging data for the prev link.
    pub prev: Option<P>,
}

impl<P: Serialize> Page<P> {
    /// Converts this page into an item collection.
    pub fn into_item_collection(
        self,
        url: &Url,
        method: &Method,
        current: P,
    ) -> Result<ItemCollection> {
        let mut item_collection = self.item_collection;
        add_link(&mut item_collection, &url, "self", current, &method)?;
        if let Some(next) = self.next {
            add_link(&mut item_collection, &url, "next", next, &method)?;
        }
        if let Some(prev) = self.prev {
            add_link(&mut item_collection, &url, "prev", prev, &method)?;
        }
        Ok(item_collection)
    }
}

fn add_link(
    item_collection: &mut ItemCollection,
    url: &Url,
    rel: &'static str,
    query: impl Serialize,
    method: &Method,
) -> Result<()> {
    match *method {
        Method::GET => {
            let mut url = url.clone();
            let mut query = serde_urlencoded::to_string(query)?;
            if let Some(existing_query) = url.query() {
                query = format!("{}&{}", existing_query, query);
            }
            if !query.is_empty() {
                url.set_query(Some(&query));
            }
            item_collection.links.push(Link::new(url, rel).geojson());
        }
        Method::POST => todo!(),
        _ => unimplemented!(), // TODO make this an error
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Page;
    use crate::assert_link;
    use http::Method;
    use stac_api::ItemCollection;
    use url::Url;

    #[test]
    fn into_item_collection_no_paging() {
        let page: Page<()> = Page {
            item_collection: ItemCollection::new(vec![]).unwrap(),
            next: None,
            prev: None,
        };
        let item_collection = page
            .into_item_collection(
                &Url::parse("http://stac-api-backend.test/").unwrap(),
                &Method::GET,
                (),
            )
            .unwrap();
        assert_eq!(item_collection.links.len(), 1);
        assert_link!(
            item_collection,
            "self",
            "http://stac-api-backend.test/",
            "application/geo+json"
        );
    }

    #[test]
    fn into_item_collection_next_get() {
        let page = Page {
            item_collection: ItemCollection::new(vec![]).unwrap(),
            next: Some([["skip", "1"], ["take", "1"]]),
            prev: None,
        };
        let item_collection = page
            .into_item_collection(
                &Url::parse("http://stac-api-backend.test/items").unwrap(),
                &Method::GET,
                [["skip", "0"], ["take", "1"]],
            )
            .unwrap();
        assert_eq!(item_collection.links.len(), 2);
        assert_link!(
            item_collection,
            "self",
            "http://stac-api-backend.test/items?skip=0&take=1",
            "application/geo+json"
        );
        assert_link!(
            item_collection,
            "next",
            "http://stac-api-backend.test/items?skip=1&take=1",
            "application/geo+json"
        );
    }

    #[test]
    fn into_item_collection_prev_get() {
        let page = Page {
            item_collection: ItemCollection::new(vec![]).unwrap(),
            prev: Some([["skip", "1"], ["take", "1"]]),
            next: None,
        };
        let item_collection = page
            .into_item_collection(
                &Url::parse("http://stac-api-backend.test/items").unwrap(),
                &Method::GET,
                [["skip", "2"], ["take", "1"]],
            )
            .unwrap();
        assert_eq!(item_collection.links.len(), 2);
        assert_link!(
            item_collection,
            "self",
            "http://stac-api-backend.test/items?skip=2&take=1",
            "application/geo+json"
        );
        assert_link!(
            item_collection,
            "prev",
            "http://stac-api-backend.test/items?skip=1&take=1",
            "application/geo+json"
        );
    }

    #[test]
    fn into_item_collection_next_get_with_params() {
        let page = Page {
            item_collection: ItemCollection::new(vec![]).unwrap(),
            next: Some([["skip", "1"], ["take", "1"]]),
            prev: None,
        };
        let item_collection = page
            .into_item_collection(
                &Url::parse("http://stac-api-backend.test/items?limit=42").unwrap(),
                &Method::GET,
                [["skip", "0"], ["take", "1"]],
            )
            .unwrap();
        assert_eq!(item_collection.links.len(), 2);
        assert_link!(
            item_collection,
            "self",
            "http://stac-api-backend.test/items?limit=42&skip=0&take=1",
            "application/geo+json"
        );
        assert_link!(
            item_collection,
            "next",
            "http://stac-api-backend.test/items?limit=42&skip=1&take=1",
            "application/geo+json"
        );
    }
}
