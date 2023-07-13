use clap::Parser;
use stac_api_backend::{MemoryBackend, PgstacBackend};
use stac_server_cli::{BackendConfig, Config};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Cli {
    /// Server configuration.
    ///
    /// If not provided, a very simple default configuration will be used.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// The address to serve the API.
    ///
    /// This will override any address configuration in the config file.
    #[arg(short, long)]
    addr: Option<String>,

    /// The address of the pgstac database.
    ///
    /// This will override any backend configuration in the config file.
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
        Config::default()
    };

    if let Some(addr) = &cli.addr {
        config.server.addr = addr.to_string();
    }
    if let Some(pgstac) = &cli.pgstac {
        config.backend.set_pgstac_config(pgstac);
    }

    match config.backend {
        BackendConfig::Memory => {
            let mut backend = MemoryBackend::new();
            stac_server_cli::load_hrefs(&mut backend, cli.hrefs)
                .await
                .unwrap();
            println!("Serving on http://{}", config.server.addr);
            stac_server::serve(backend, config.server).await.unwrap()
        }
        BackendConfig::Pgstac(pgstac) => {
            let (_, _) = tokio_postgres::connect(&pgstac.config, tokio_postgres::NoTls)
                .await
                .unwrap();
            let mut backend = PgstacBackend::connect(&pgstac.config).await.unwrap();
            stac_server_cli::load_hrefs(&mut backend, cli.hrefs)
                .await
                .unwrap();
            println!("Serving on http://{}", config.server.addr);
            stac_server::serve(backend, config.server).await.unwrap()
        }
    };
}
