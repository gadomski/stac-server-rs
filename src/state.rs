use crate::Config;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub pool: Pool<PostgresConnectionManager<NoTls>>,
    pub config: Config,
}
