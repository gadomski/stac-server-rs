use crate::{Error, Result};
use stac::Collection;
use url::Url;

/// Build hrefs.
#[derive(Debug)]
pub struct Hrefs {
    root: Option<Url>,
}

impl Hrefs {
    /// Creates a new href builder, rooted at `addr`.
    ///
    /// If `addr` is none, all hrefs will just be paths, e.g. `/collections.`
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Hrefs;
    /// # use url::Url;
    /// let hrefs = Hrefs::new(None);
    /// assert_eq!(hrefs.root().unwrap(), "/");
    ///
    /// let root = Url::parse("http://stac-api-rs.test").unwrap();
    /// let hrefs = Hrefs::new(root);
    /// assert_eq!(hrefs.root().unwrap(), "http://stac-api-rs.test/");
    /// ```
    pub fn new(root: impl Into<Option<Url>>) -> Hrefs {
        // TODO validate that we can join the root, so we can return Strings,
        // not Results, from the methods.
        Hrefs { root: root.into() }
    }

    /// Returns the root href.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Hrefs;
    /// let hrefs = Hrefs::new(None);
    /// assert_eq!(hrefs.root().unwrap(), "/");
    /// ```
    pub fn root(&self) -> Result<String> {
        self.href("")
    }

    /// Returns an href.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Hrefs;
    /// # use url::Url;
    /// let root = Url::parse("http://stac-api-rs.test/api/stac/v1/").unwrap();
    /// let hrefs = Hrefs::new(root);
    /// assert_eq!(
    ///     hrefs.href("some/sub/path").unwrap(),
    ///     "http://stac-api-rs.test/api/stac/v1/some/sub/path"
    /// );
    /// ```
    pub fn href(&self, path: &str) -> Result<String> {
        if let Some(root) = self.root.as_ref() {
            root.join(path)
                .map(|url| url.to_string())
                .map_err(Error::from)
        } else {
            Ok(format!("/{}", path))
        }
    }

    /// Returns a collection's href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Collection;
    /// # use stac_api::Hrefs;
    /// let hrefs = Hrefs::new(None);
    /// assert_eq!(
    ///     hrefs.collection(&Collection::new("an-id", "a description")).unwrap(),
    ///     "/collections/an-id"
    /// );
    pub fn collection(&self, collection: &Collection) -> Result<String> {
        // TODO Reduce number of strings we're making
        self.href(&format!("collections/{}", collection.id))
    }
}
