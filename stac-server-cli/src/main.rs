use axum::Server;
use clap::{Parser, Subcommand};
use pgstac::PgstacBackend;
use stac_backend::MemoryBackend;
use stac_server::Config;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Parser)]
struct Cli {
    config: PathBuf,
    addr: Option<String>,

    #[command(subcommand)]
    backend: Backend,
}

#[derive(Debug, Subcommand)]
enum Backend {
    Memory { paths: Vec<PathBuf> },
    Pgstac { connection: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut config = Config::from_toml(cli.config).await.unwrap();
    let addr = if let Some(addr) = &cli.addr {
        config.addr = Some(addr.to_string());
        addr
    } else if let Some(addr) = &config.addr {
        addr
    } else {
        panic!("addr must be provided on the command line or in the config");
    };
    let addr = addr.parse::<SocketAddr>().unwrap();
    let router = match cli.backend {
        Backend::Memory { paths } => {
            let mut backend = MemoryBackend::new();
            stac_server_cli::load_files_into_memory_backend(&mut backend, &paths)
                .await
                .unwrap();
            stac_server::api(backend, config)
        }
        Backend::Pgstac { connection } => {
            let backend = PgstacBackend::from_str(&connection).await.unwrap();
            stac_server::api(backend, config)
        }
    };
    println!("Serving on http://{}", addr);
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
