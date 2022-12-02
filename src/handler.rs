use crate::{extract::LinkBuilder, ApiState, Backend};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use stac::{Catalog, Collection};

pub async fn landing_page<B: Backend>(
    State(state): State<ApiState<B>>,
    link_builder: LinkBuilder,
) -> Result<Json<Catalog>, (StatusCode, String)> {
    let mut catalog = Catalog::new(state.config.catalog.id);
    catalog.description = state.config.catalog.description;
    catalog.links.push(link_builder.self_());
    catalog.links.push(link_builder.root());
    catalog.additional_fields.insert(
        "conformsTo".to_string(),
        vec!["https://api.stacspec.org/v1.0.0-rc.2/core".to_string()].into(),
    );
    let collections = state.backend.collections().await.map_err(internal_error)?;
    for collection in collections {
        catalog.links.push(link_builder.collection(collection));
    }

    Ok(Json(catalog))
}

pub async fn collection<B: Backend>(
    Path(collection_id): Path<String>,
    State(state): State<ApiState<B>>,
) -> Result<Json<Collection>, (StatusCode, String)> {
    let collection = state
        .backend
        .collection(&collection_id)
        .await
        .map_err(internal_error)?;
    Ok(Json(collection))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
