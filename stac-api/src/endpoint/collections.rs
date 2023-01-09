use crate::{Backend, Hrefs, Result};
use serde::Serialize;
use stac::{Collection, Link, Links};

#[derive(Debug, Serialize)]
pub struct Collections {
    pub collections: Vec<Collection>,
    pub links: Vec<Link>,
}

impl Collections {
    pub async fn new(backend: impl Backend, hrefs: Hrefs) -> Result<Collections> {
        let collections = backend.collections().await.unwrap();
        let links = vec![
            Link::root(hrefs.root()?),
            Link::self_(hrefs.href("collections")?),
        ];
        Ok(Collections { collections, links })
    }
}

impl Links for Collections {
    fn links(&self) -> &[Link] {
        &self.links
    }

    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

#[cfg(test)]
mod tests {
    use super::Collections;
    use crate::{Hrefs, MemoryBackend};
    use stac::Links;

    #[tokio::test]
    async fn collections() {
        let collections = Collections::new(MemoryBackend::new(), Hrefs::new(None))
            .await
            .unwrap();

        let root_link = collections.root_link().unwrap();
        assert_eq!(root_link.href, "/");
        let self_link = collections.self_link().unwrap();
        assert_eq!(self_link.href, "/collections");

        assert!(collections.collections.is_empty());
    }
}
