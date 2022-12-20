use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Expected one argument");
        std::process::exit(1);
    }
    let _ = tokio_postgres::connect(&args[1], NoTls).await.unwrap();
}
