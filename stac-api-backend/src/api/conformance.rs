use super::Api;
use crate::{Backend, Error};
use stac_api::{
    Conformance, COLLECTIONS_URI, CORE_URI, FEATURES_URI, GEOJSON_URI, OGC_API_FEATURES_URI,
};

impl<B> Api<B>
where
    B: Backend,
    Error: From<<B as Backend>::Error>,
{
    /// Returns the conformance structure.
    pub fn conformance(&self) -> Conformance {
        let mut conforms_to = vec![CORE_URI.to_string()];
        if self.features {
            conforms_to.extend([
                FEATURES_URI.to_string(),
                COLLECTIONS_URI.to_string(),
                OGC_API_FEATURES_URI.to_string(),
                GEOJSON_URI.to_string(),
            ])
        }
        Conformance { conforms_to }
    }
}
