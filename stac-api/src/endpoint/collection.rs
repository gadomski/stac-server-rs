use crate::{Backend, Hrefs, Result};
use serde::Serialize;
use stac::{Link, Links};

#[derive(Debug, Serialize)]
pub struct Collection(stac::Collection);

impl Collection {
    pub async fn new(backend: impl Backend, id: &str, hrefs: Hrefs) -> Result<Option<Collection>> {
        if let Some(mut collection) = backend.collection(id).await? {
            let root = hrefs.root()?;
            collection.set_link(Link::root(&root));
            collection.set_link(Link::parent(root));
            collection.set_link(Link::self_(hrefs.collection(&collection)?));
            Ok(Some(Collection(collection)))
        } else {
            Ok(None)
        }
    }
}

impl Links for Collection {
    fn links(&self) -> &[Link] {
        self.0.links()
    }

    fn links_mut(&mut self) -> &mut Vec<Link> {
        self.0.links_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::Collection;
    use crate::{Backend, Hrefs, MemoryBackend};
    use stac::Links;

    #[tokio::test]
    async fn collections() {
        let mut backend = MemoryBackend::new();
        backend
            .add_collection(stac::Collection::new("an-id", "a description"))
            .await
            .unwrap();
        let collection = Collection::new(backend, "an-id", Hrefs::new(None))
            .await
            .unwrap()
            .unwrap();

        let root_link = collection.root_link().unwrap();
        assert_eq!(root_link.href, "/");
        let parent_link = collection.parent_link().unwrap();
        assert_eq!(parent_link.href, "/");
        let self_link = collection.self_link().unwrap();
        assert_eq!(self_link.href, "/collections/an-id");
    }
}
