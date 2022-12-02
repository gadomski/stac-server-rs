use crate::{extract::SelfHref, State};
use axum::{extract, http::StatusCode, response::Json};
use serde_json::Value;
use stac::{media_type, Catalog, Collection, Link};

pub async fn landing_page(
    extract::State(state): extract::State<State>,
    SelfHref(self_href): SelfHref,
) -> Result<Json<Catalog>, (StatusCode, String)> {
    let mut catalog = Catalog::new(state.config.catalog.id);
    catalog.description = state.config.catalog.description;
    let self_link = Link {
        href: self_href,
        rel: "self".to_string(),
        r#type: Some(media_type::JSON.to_string()),
        title: None,
        additional_fields: Default::default(),
    };
    let mut root_link = self_link.clone();
    root_link.rel = "root".to_string();
    catalog.links.push(self_link);
    catalog.links.push(root_link);
    catalog.additional_fields.insert(
        "conformsTo".to_string(),
        vec!["https://api.stacspec.org/v1.0.0-rc.2/core".to_string()].into(),
    );

    let connection = state.pool.get().await.map_err(internal_error)?;
    let row = connection
        .query_one("SELECT pgstac.all_collections();", &[])
        .await
        .map_err(internal_error)?;
    let collections: Value = row.get(0);
    for collection in collections
        .as_array()
        .expect("collections should be an array")
    {
        let collection: Collection =
            serde_json::from_value(collection.clone()).map_err(internal_error)?;
        catalog.links.push(Link {
            href: format!("http://localhost:3000/collections/{}", collection.id),
            rel: "child".to_string(),
            r#type: Some(media_type::GEOJSON.to_string()),
            title: collection.title,
            additional_fields: Default::default(),
        });
    }

    Ok(Json(catalog))
}

pub async fn collection(
    extract::Path(collection_id): extract::Path<String>,
    extract::State(state): extract::State<State>,
) -> Result<Json<Collection>, (StatusCode, String)> {
    let connection = state.pool.get().await.map_err(internal_error)?;
    let row = connection
        .query_one(
            "SELECT * FROM pgstac.get_collection($1);",
            &[&collection_id],
        )
        .await
        .map_err(internal_error)?;
    let value: Value = row.try_get(0).map_err(internal_error)?;
    let collection: Collection = serde_json::from_value(value).map_err(internal_error)?;
    Ok(Json(collection))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
