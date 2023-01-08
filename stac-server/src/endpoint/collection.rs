use crate::{extract::Hrefs, Result, State};
use axum::{
    extract::{self, Path},
    Json,
};
use serde::Serialize;
use stac::{Link, Links};
use stac_api::Backend;

#[derive(Debug, Serialize)]
pub struct Collection(stac::Collection);

impl Collection {
    async fn new(backend: impl Backend, id: &str, hrefs: Hrefs) -> Result<Option<Collection>> {
        if let Some(mut collection) = backend.collection(id).await? {
            collection.set_link(Link::root(hrefs.root()));
            collection.set_link(Link::parent(hrefs.root()));
            collection.set_link(Link::self_(hrefs.collection(&collection)));
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

pub async fn collection<B: Backend>(
    extract::State(state): extract::State<State<B>>,
    Path(id): Path<String>,
    hrefs: Hrefs,
) -> Json<Collection> {
    // TODO handle error pages
    Json(
        Collection::new(state.backend, &id, hrefs)
            .await
            .unwrap()
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::Collection;
    use crate::extract::Hrefs;
    use stac::Links;
    use stac_api::{Backend, MemoryBackend};

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
