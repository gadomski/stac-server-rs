mod api;
mod conformance;
mod features;
mod root;

pub use api::Api;

/// The default media type for the `service-desc` links.
pub const DEFAULT_SERVICE_DESC_MEDIA_TYPE: &str = "application/vnd.oai.openapi+json;version=3.1";

#[cfg(all(test, feature = "memory"))]
mod tests {
    use super::Api;
    use crate::memory::MemoryBackend;
    use stac::Catalog;

    pub(crate) fn api() -> Api<MemoryBackend> {
        Api::new(
            MemoryBackend::new(),
            Catalog::new("test-catalog", "A catalog for testing"),
            "http://stac-api-backend.test",
        )
        .unwrap()
    }
}
