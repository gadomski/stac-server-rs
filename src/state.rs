use crate::{Backend, Config};

#[derive(Clone, Debug)]
pub struct ApiState<B: Backend> {
    pub config: Config,
    pub backend: B,
}
