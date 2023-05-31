mod service;
mod api;
mod store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "[::1]:8800".parse().unwrap();

    let service = service::Service::with_mdbx("/tmp/skunkr/mdbx-2");

    tonic::transport::Server::builder()
        .add_service(api::data_server::DataServer::new(service))
        .add_service(api::create_reflection_server()?)
        .serve(address)
        .await?;

    Ok(())
}

