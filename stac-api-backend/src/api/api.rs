use crate::{Backend, Error, Result, DEFAULT_SERVICE_DESC_MEDIA_TYPE};
use stac::Catalog;
use stac_api::UrlBuilder;

/// A structure for generating STAC API endpoints.
#[derive(Clone, Debug)]
pub struct Api<B: Backend> {
    /// The backend for this API.
    pub backend: B,

    /// The url builder for this api.
    pub url_builder: UrlBuilder,

    /// If true, this API will include links for the [Features](https://github.com/radiantearth/stac-api-spec/tree/main/ogcapi-features) endpoints.
    ///
    /// We don't support _just_ collections.
    pub features: bool,

    /// The media type for the `service-desc` endpoint.
    ///
    /// Defaults to [DEFAULT_SERVICE_DESC_MEDIA_TYPE].
    pub service_desc_media_type: String,

    /// The base catalog for this api.
    pub catalog: Catalog,
}

impl<B: Backend> Api<B>
where
    Error: From<<B as Backend>::Error>,
{
    /// Creates a new endpoint generator with the given backend, catalog, and root url.
    ///
    /// The catalog is used as the root endpoint. By default, the API will
    /// include links for
    /// [Features](https://github.com/radiantearth/stac-api-spec/tree/main/ogcapi-features)
    /// -- set `features` to `False` to disable this behavior.
    pub fn new(backend: B, catalog: Catalog, url: &str) -> Result<Api<B>> {
        Ok(Api {
            backend,
            catalog,
            features: true,
            service_desc_media_type: DEFAULT_SERVICE_DESC_MEDIA_TYPE.to_string(),
            url_builder: UrlBuilder::new(url)?,
        })
    }

    /// Sets the value of `features`.
    pub fn features(mut self, features: bool) -> Api<B> {
        self.features = features;
        self
    }
}
