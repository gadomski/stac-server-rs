use axum::{routing::get, Router, Server};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use clap::Parser;
use stac_server::{
    handler::{collection, landing_page},
    Config, State,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use tokio_postgres::{Config as PostgresConfig, NoTls};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    config_file: PathBuf,
    #[arg(env)]
    postgres_user: String,
    #[arg(env)]
    postgres_pass: String,
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
    let config: Config = toml::from_str(&config).unwrap();
    let mut postgres_config = PostgresConfig::new();
    postgres_config
        .user(&cli.postgres_user)
        .password(&cli.postgres_pass)
        .host(&cli.postgres_host)
        .dbname(&cli.postgres_dbname)
        .port(cli.postgres_port);
    let manager = PostgresConnectionManager::new(postgres_config, NoTls);
    let pool = Pool::builder().build(manager).await.unwrap();
    let state = State {
        pool,
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
