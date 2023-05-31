
use std::pin::Pin;
use tonic::{Request, Response, Status};
use tokio::sync::{mpsc, oneshot};
use tonic::codegen::futures_core::Stream;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::api::{GetRequest,GetResponse,SetRequest,SetResponse,ScanRequest,ScanResponse};
use super::api::data_server::Data;

use super::store::*;

pub struct Service {
    pub store: Box<dyn KeyValueStore + Send + Sync>
}

impl Service {
    pub fn with_mdbx(path: &str) -> Self {
        Service {
            store: Box::new(MdbxStore::new(path))
        }
    }
}

#[tonic::async_trait]
impl Data for Service {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let r = request.into_inner();
        match &self.store.get(r.key.as_str()) {
            Some(value) => Ok(Response::new(GetResponse {
                value: value.to_string()
            })),
            None => Ok(Response::new(GetResponse{ // TODO: error responses
                value: String::from("")
            })),
        }

    }

    async fn set(&self, request: Request<SetRequest>) -> Result<Response<SetResponse>, Status> {
        let r = request.into_inner();
        let inserted = &self.store.set(r.key.as_str(), r.value.as_str());
        Ok(Response::new(SetResponse {
            success: *inserted
        }))
    }

    type ScanStream = Pin<Box<dyn Stream<Item = Result<ScanResponse, Status>> + Send>>;

    async fn scan(&self, request: Request<ScanRequest>, ) -> Result<Response<Self::ScanStream>, Status> {

        let req = request.into_inner();

        let from = match req.from.as_str() {
            "" => None,
            s => Some(s.to_string())
        };

        // inbound data channel ( db -> service )
        let (source, mut receiver) = mpsc::channel(10);

        let (filter_sender, filter_receiver) = oneshot::channel();
        filter_sender.send(from).expect("failed to send filter command");

        // start the scanner
        let _ = &self.store.scan(filter_receiver, source);

        // outbound channel ( service -> caller )
        let (dispatcher, sink) = mpsc::unbounded_channel();

        // receive, transform, and relay the rows
        tokio::spawn(async move {
            loop {
                let _ = match receiver.recv().await {
                    Some(kv) => dispatcher.send(Ok(ScanResponse { key: kv.0, value: kv.1 })),
                    None => break
                };
            }
        });

        let stream = UnboundedReceiverStream::new(sink);
        Ok(Response::new(Box::pin(stream) as Self::ScanStream))
    }
}