mod service;
mod api;
mod store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let port = args.port;
    let address = format!("[::1]:{}", port).parse().unwrap();

    let mut data = args.data;
    match (args.leveldb, args.mdbx) {
        (true,false) => {
            // data = format!("{}/leveldb", data)
            println!("leveldb not implemented");
            process::exit(1);
        },
        _ => data = format!("{}/mdbx", data)
    }

    let service = service::Service::with_mdbx(&data);

    tonic::transport::Server::builder()
        .add_service(api::data_server::DataServer::new(service))
        .add_service(api::create_reflection_server()?)
        .serve(address)
        .await?;

    Ok(())
}

use std::process;
use clap::Parser;
use clap::ArgGroup;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("storage")
        .required(true)
        .args(&["mdbx", "leveldb"]),
))]
struct Args {
    /// Path to data
    #[arg(short, long)]
    data: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8800)]
    port: u16,

    #[arg(long)]
    mdbx: bool,

    #[arg(long)]
    leveldb: bool
}
