use axum::Server;
use clap::Parser;
use stac_api_backend::MemoryBackend;
use stac_server::Config;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Parser)]
struct Cli {
    /// Server configuration.
    ///
    /// If not provided, a very simple default configuration will be used.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// The address to serve the API.
    ///
    /// If not provided, will be read from the configuration.
    #[arg(short, long)]
    addr: Option<String>,

    /// The address of the pgstac database.
    ///
    /// If not provided, a memory backend will be used.
    #[arg(short, long)]
    pgstac: Option<String>,

    /// The hrefs of STAC collections and items to read and load into the
    /// backend when starting the server.
    hrefs: Vec<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut config = if let Some(config) = cli.config {
        Config::from_toml(config).await.unwrap()
    } else {
        stac_server_cli::default_config()
    };
    if let Some(addr) = &cli.addr {
        config.addr = addr.to_string();
    }
    let addr = config.addr.parse::<SocketAddr>().unwrap();
    let router = if let Some(_) = cli.pgstac {
        unimplemented!()
    } else {
        let mut backend = MemoryBackend::new();
        stac_server_cli::load_files_into_backend(&mut backend, &cli.hrefs).await;
        stac_server::api(backend, config).unwrap()
    };
    println!("Serving on http://{}", addr);
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
