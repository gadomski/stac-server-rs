use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Host, OriginalUri},
    http::{request::Parts, Uri},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use stac::{media_type, Collection, Link};

#[derive(Debug)]
pub struct LinkBuilder {
    host: String,
    original_uri: Uri,
}

impl LinkBuilder {
    pub fn self_(&self) -> Link {
        Link {
            href: format!("http://{}{}", self.host, self.original_uri),
            rel: "self".to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: None,
            additional_fields: Default::default(),
        }
    }

    pub fn root(&self) -> Link {
        // TODO this should be able to adapt to mounting points.
        Link {
            href: format!("http://{}/", self.host),
            rel: "root".to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: None,
            additional_fields: Default::default(),
        }
    }

    pub fn collection(&self, collection: Collection) -> Link {
        Link {
            href: format!("http://{}/collections/{}", self.host, collection.id),
            rel: "root".to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: collection.title,
            additional_fields: Default::default(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for LinkBuilder
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
        Ok(LinkBuilder { host, original_uri })
    }
}
