use crate::{extract::HrefBuilder, ApiState, Backend};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use stac::{media_type, Catalog, Collection, Link, Links};

#[derive(Debug, Serialize, Deserialize)]
pub struct LandingPage {
    #[serde(flatten)]
    pub catalog: Catalog,

    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionsPage {
    pub links: Vec<Link>,
    pub collections: Vec<Collection>,
}

impl Links for CollectionsPage {
    fn links(&self) -> &[Link] {
        &self.links
    }
    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

pub async fn landing_page<B: Backend>(
    href_builder: HrefBuilder,
    State(state): State<ApiState<B>>,
) -> Result<Json<LandingPage>, (StatusCode, String)> {
    let mut catalog = state.config.catalog.to_catalog();
    let root_href = href_builder.root();
    catalog.set_root_link(root_href.clone(), catalog.title.clone());
    catalog.set_self_link(root_href, catalog.title.clone());
    for collection in state.backend.collections().await.map_err(internal_error)? {
        let mut link = Link::child(href_builder.collection(&collection));
        link.title = collection.title;
        catalog.links.push(link);
    }
    if state.config.ogc_api_features {
        catalog.links.push(Link {
            href: href_builder.href("conformance"),
            rel: "conformance".to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: None,
            additional_fields: Default::default(),
        });
        catalog.links.push(Link {
            href: href_builder.href("collections"),
            rel: "data".to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: None,
            additional_fields: Default::default(),
        });
    }
    let landing_page = LandingPage {
        catalog,
        conforms_to: state.config.conforms_to(),
    };
    Ok(Json(landing_page))
}

pub async fn collections<B: Backend>(
    href_builder: HrefBuilder,
    State(state): State<ApiState<B>>,
) -> Result<Json<CollectionsPage>, (StatusCode, String)> {
    let mut links = Vec::new();
    let mut root_link = Link::root(href_builder.root());
    root_link.title = state.config.catalog.title.clone();
    links.push(root_link);
    links.push(Link {
        href: href_builder.href("collections"),
        rel: "self".to_string(),
        r#type: Some(media_type::JSON.to_string()),
        title: None,
        additional_fields: Default::default(),
    });
    let collections = state.backend.collections().await.map_err(internal_error)?;
    Ok(Json(CollectionsPage { links, collections }))
}

pub async fn collection<B: Backend>(
    href_builder: HrefBuilder,
    State(state): State<ApiState<B>>,
    Path(id): Path<String>,
) -> Result<Json<Collection>, (StatusCode, String)> {
    if let Some(mut collection) = state
        .backend
        .collection(&id)
        .await
        .map_err(internal_error)?
    {
        collection.set_root_link(href_builder.root(), state.config.catalog.title.clone());
        collection.links.push(Link {
            href: href_builder.root(),
            rel: "parent".to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: state.config.catalog.title.clone(),
            additional_fields: Default::default(),
        }); // TODO add set_parent_link()
        collection.set_self_link(
            href_builder.collection(&collection),
            collection.title.clone(),
        );
        Ok(Json(collection))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("no collection with id: {}", id),
        ))
    }
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
