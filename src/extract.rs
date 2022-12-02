use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Host, OriginalUri},
    http::request::Parts,
    response::{IntoResponse, Response},
    RequestPartsExt,
};

#[derive(Debug)]
pub struct SelfHref(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for SelfHref
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let Host(host) = parts
            .extract::<Host>()
            .await
            .map_err(|err| err.into_response())?;
        let OriginalUri(original_uri) = parts.extract::<OriginalUri>().await.unwrap();
        Ok(SelfHref(format!("http://{}{}", host, original_uri)))
    }
}
