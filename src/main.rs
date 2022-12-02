use axum::{routing::get, Router, Server};
use clap::Parser;
use stac_server::{
    backend::PgstacBackend,
    handler::{collection, landing_page},
    ApiConfig, ApiState,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use tokio_postgres::Config;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    config_file: PathBuf,
    #[arg(env)]
    postgres_user: String,
    #[arg(env)]
    postgres_password: String,
    #[arg(env)]
    postgres_dbname: String,
    #[arg(env)]
    postgres_host: String,
    #[arg(env)]
    postgres_port: u16,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut reader = BufReader::new(File::open(cli.config_file).unwrap());
    let mut config = String::new();
    reader.read_to_string(&mut config).unwrap();
    let config: ApiConfig = toml::from_str(&config).unwrap();
    let mut postgres_config = Config::new();
    postgres_config
        .user(&cli.postgres_user)
        .password(&cli.postgres_password)
        .host(&cli.postgres_host)
        .dbname(&cli.postgres_dbname)
        .port(cli.postgres_port);
    let backend = PgstacBackend::new(postgres_config).await.unwrap();
    let state = ApiState {
        backend,
        config: config.clone(),
    };
    let app = Router::new()
        .route("/", get(landing_page))
        .route("/collections/:collection_id", get(collection))
        .with_state(state.clone());
    Server::bind(&config.addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
