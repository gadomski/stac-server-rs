use crate::{ApiConfig, Backend};

#[derive(Debug, Clone)]
pub struct ApiState<B: Backend> {
    pub backend: B,
    pub config: ApiConfig,
}
