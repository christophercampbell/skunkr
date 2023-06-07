mod api;
mod service;
mod store;

use chrono::Local;
use env_logger;
use log;
use std::io::Write;
use std::net::SocketAddr;
use std::process;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    init_logging();

    let args = Args::from_args();

    let data = match args.data_path() {
        Ok(path) => path,
        Err(msg) => {
            log::error!("{}", msg);
            process::exit(1);
        }
    };

    tonic::transport::Server::builder()
        .add_service(
            api::data_server::DataServer::new(service::Service::new(&data))
        ).add_service(api::create_reflection_server()?)
        .serve(args.bind_addr())
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

impl Args {
    fn bind_addr(&self) -> SocketAddr {
        let port = &self.port;
        return format!("0.0.0.0:{}", port).parse().unwrap();
    }
    fn data_path(&self) -> Result<String,String> {
        match self.data_dir_name() {
            "leveldb" => Err(String::from("leveldb not implemented")),
            x => Ok(format!("{}/{}", self.data, x))
        }
    }
    fn data_dir_name(&self) -> &'static str {
        match (self.mdbx, self.leveldb) {
            (false,true) => "leveldb",
            _ => "mdbx"
        }
    }
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
