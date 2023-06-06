mod api;
mod service;
mod store;

use chrono::Local;
use env_logger;
use log;
use std::io::Write;
use std::process;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let args = Args::from_args();

    let port = args.port;
    let address = format!("0.0.0.0:{}", port).parse().unwrap();
    let mut data = args.data;

    if args.leveldb {
        // data = format!("{}/leveldb", data)
        log::error!("leveldb not implemented");
        process::exit(1);
    }
    if args.mdbx || !args.mdbx { // default is mdbx
        data = format!("{}/mdbx", data);
    }

    let service = service::Service::start(&data);

    tonic::transport::Server::builder()
        .add_service(api::data_server::DataServer::new(service))
        .add_service(api::create_reflection_server()?)
        .serve(address)
        .await?;

    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Args", about = "Program arguments")]
pub struct Args {
    #[structopt(short = "d", long = "data", required = true)]
    data: String,

    #[structopt(short = "p", long = "port", default_value = "7070")]
    port: u16,

    #[structopt(long = "mdbx", conflicts_with = "leveldb")]
    mdbx: bool,

    #[structopt(long = "leveldb", conflicts_with = "mdbx")]
    leveldb: bool,
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
        }).init();
}
