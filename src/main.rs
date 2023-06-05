mod service;
mod api;
mod store;

use clap::ArgGroup;
use clap::Parser;
use chrono::Local;
use env_logger;
use log;
use std::process;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    init_logging();

    let args = Args::parse();

    let port = args.port;
    let address = format!("[::1]:{}", port).parse().unwrap();

    let mut data = args.data;
    match (args.leveldb, args.mdbx) {
        (true,false) => {
            // data = format!("{}/leveldb", data)
            log::error!("leveldb not implemented");
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

/// Use env_logger env variables, as in RUST_LOG=info, or for module level RUST_LOG=error,skunkr::service=warn
fn init_logging() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();
            let color = match level {
                log::Level::Error => "\x1b[31m",   // Red
                log::Level::Warn => "\x1b[33m",    // Yellow
                log::Level::Info => "\x1b[32m",    // Green
                log::Level::Debug => "\x1b[34m",   // Blue
                log::Level::Trace => "\x1b[35m",   // Magenta
            };
            let reset = "\x1b[0m";
            writeln!(
                buf,
                "{} {}[{}]{} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                color,
                level,
                reset,
                record.args()
            )
        })
        .init();
}
