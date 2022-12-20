use pgstac::PgstacBackend;
use stac_backend::Backend;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Expected one argument");
        std::process::exit(1);
    }
    let backend = PgstacBackend::from_str(&args[1]).await.unwrap();
    for collection in backend.collections().await.unwrap() {
        println!("{}", collection.id);
    }
}
