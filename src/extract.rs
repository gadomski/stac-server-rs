use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Host},
    http::request::Parts,
    response::Response,
    RequestPartsExt,
};
use stac::Collection;

pub struct HrefBuilder {
    host: Option<String>,
}

impl HrefBuilder {
    pub fn root(&self) -> String {
        if let Some(host) = &self.host {
            format!("http://{}/", host) // TODO check http/https
                                        // TODO allow for mounting points
        } else {
            "/".to_string()
        }
    }

    pub fn collection(&self, collection: &Collection) -> String {
        self.href(&format!("collections/{}", collection.id))
    }

    pub fn href(&self, s: &str) -> String {
        let mut href = self.root();
        href.push_str(s);
        href
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for HrefBuilder
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let host = parts.extract::<Host>().await.ok().map(|Host(host)| host);
        Ok(HrefBuilder { host })
    }
}
