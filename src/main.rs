use axum::Server;
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json::Value;
use stac::{Collection, Item};
use stac_server::{
    backend::{Pgstac, Simple},
    Config, Error,
};
use std::path::PathBuf;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use url::Url;

#[derive(Debug, Parser)]
struct Cli {
    addr: String,
    config_file: PathBuf,
    #[arg(long)]
    href: Vec<String>,
    #[command(subcommand)]
    backend: Backend,
}

#[derive(Debug, Subcommand)]
enum Backend {
    Simple {},
    Pgstac {
        user: String,
        password: String,
        dbname: String,
        host: String,
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let config = Config::from_toml(cli.config_file)?;
    match cli.backend {
        Backend::Simple {} => {
            let mut backend = Simple::new();
            if !cli.href.is_empty() {
                populate_backend(&mut backend, &cli.href).await?;
            }
            let api = stac_server::api(config, backend);
            Server::bind(&cli.addr.parse()?)
                .serve(api.into_make_service())
                .await?;
        }
        Backend::Pgstac {
            user,
            password,
            dbname,
            host,
            port,
        } => {
            let mut postgres_config = tokio_postgres::Config::new();
            postgres_config
                .user(&user)
                .password(password)
                .dbname(&dbname)
                .host(&host)
                .port(port);
            let mut backend = Pgstac::new(postgres_config).await?;
            if !cli.href.is_empty() {
                populate_backend(&mut backend, &cli.href).await?;
            }
            let api = stac_server::api(config, backend);
            Server::bind(&cli.addr.parse()?)
                .serve(api.into_make_service())
                .await?;
        }
    }
    Ok(())
}

async fn populate_backend<B: stac_server::Backend>(
    backend: &mut B,
    hrefs: &[String],
) -> Result<(), Error> {
    // TODO use async threads
    let client = Client::new();
    let mut collections: Vec<Collection> = Vec::new();
    let mut items: Vec<Item> = Vec::new();
    for href in hrefs {
        // TODO add async reads to stac-rs
        // TODO support following links
        let mut value: Value = if let Ok(url) = Url::parse(&href) {
            client.get(url).send().await?.json().await?
        } else {
            let mut reader = File::open(href).await.map(|file| BufReader::new(file))?;
            let mut string = String::new();
            reader.read_to_string(&mut string).await?;
            serde_json::from_str(&string)?
        };
        // TODO add item collections to stac-rs
        if let Some(r#type) = value.get("type").and_then(|r#type| r#type.as_str()) {
            match r#type {
                "Collection" => collections.push(serde_json::from_value(value)?),
                "Feature" => items.push(serde_json::from_value(value)?),
                "FeatureCollection" => {
                    if let Some(Value::Array(features)) = value.get_mut("features").take() {
                        for feature in features {
                            items.push(serde_json::from_value(feature.take())?);
                        }
                    } else {
                        unimplemented!()
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
    for collection in collections {
        backend.add_collection(collection).await?;
    }
    for item in items {
        backend.add_item(item).await?;
    }
    Ok(())
}
